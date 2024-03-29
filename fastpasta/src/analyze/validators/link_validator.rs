//! Contains the [LinkValidator] that contains all the [RDH] subvalidators, and delegates all checks for a specific link.
//!
//! A [LinkValidator] is created for each link that is being checked.
//! The [LinkValidator] is responsible for creating and running all the [RDH] subvalidators, and delegating payload depending on target system.
//! It also contains a [ConstGenericRingBuffer] that is used to store the previous two [RDH]s, to be able to include them in error messages.
//!
//! Adding a new system to the validator is done by adding a new module to the [validators](crate::analyze::validators) module, and adding the new system to the [System] enum.
//! The new module should contain a main payload validator that can be used by the [LinkValidator] to delegate payload to.
//! Unfortunately it cannot be implemented through trait objects as they cannot be stored in the [LinkValidator] without using dynamic traits.
//!
//! In the `do_checks` function, the [LinkValidator] will delegate the payload to the correct validator depending on the target system.
//! The new system should be added to the match statement, along with how to delegate the payload to the new validator.

use crate::util::*;

/// Main validator that handles all checks on a specific link.
///
/// A [LinkValidator] is created for each link that is being checked.
pub struct LinkValidator<T: RDH, C: ChecksOpt + FilterOpt + CustomChecksOpt + UtilOpt + 'static> {
    config: &'static C,
    running_checks: bool,
    /// Producer channel to send stats through.
    stats_send: flume::Sender<StatType>,
    /// Consumer channel to receive data from.
    data_recv_chan: crossbeam_channel::Receiver<CdpTuple<T>>,
    its_cdp_validator: CdpRunningValidator<T, C>,
    rdh_running_validator: RdhCruRunningChecker<T>,
    rdh_sanity_validator: RdhCruSanityValidator<T>,
    prev_rdhs: ConstGenericRingBuffer<T, 2>,
}

type CdpTuple<T> = (T, Vec<u8>, u64);

impl<T: RDH, C: 'static + ChecksOpt + FilterOpt + CustomChecksOpt + UtilOpt> LinkValidator<T, C> {
    /// Creates a new [LinkValidator] and the [StatType] sender channel to it, from a config that implements [ChecksOpt] + [FilterOpt].
    ///
    /// The sender channel is unbounded
    pub fn new(
        global_config: &'static C,
        stats_send_chan: flume::Sender<StatType>,
    ) -> (Self, crossbeam_channel::Sender<CdpTuple<T>>) {
        let rdh_sanity_validator = RdhCruSanityValidator::new_from_config(global_config);

        let (data_send, data_recv) = crossbeam_channel::unbounded();
        (
            Self {
                config: global_config,
                running_checks: match global_config.check().unwrap() {
                    CheckCommands::All(_) => true,
                    CheckCommands::Sanity(_) => false,
                },

                stats_send: stats_send_chan.clone(),
                data_recv_chan: data_recv,
                its_cdp_validator: its::cdp_running::CdpRunningValidator::new(
                    global_config,
                    stats_send_chan,
                ),
                rdh_running_validator: RdhCruRunningChecker::default(),
                rdh_sanity_validator,
                prev_rdhs: ConstGenericRingBuffer::<_, 2>::new(),
            },
            data_send,
        )
    }

    /// Creates a new [LinkValidator] and the [StatType] sender channel with a specified capacity, from a config that implements [ChecksOpt] + [FilterOpt].
    ///
    /// Creates an unbounded channel if capacity is [None]
    pub fn with_chan_capacity(
        global_config: &'static C,
        stats_send_chan: flume::Sender<StatType>,
        chan_capacity: Option<usize>,
    ) -> (Self, crossbeam_channel::Sender<CdpTuple<T>>) {
        let rdh_sanity_validator = RdhCruSanityValidator::new_from_config(global_config);

        // Capacity of the channel (FIFO) to Link Validator threads in terms of CDPs (RDH, Payload, Memory position)
        //
        // Larger capacity means less overhead, but more memory usage
        // Too small capacity will cause the producer thread to block
        // Too large capacity CAN cause the consumer thread to block, this scenario is more likely if there's fewer link validator threads
        let (data_send, data_recv) = match chan_capacity {
            Some(cap) => crossbeam_channel::bounded(cap),
            None => crossbeam_channel::unbounded(),
        };

        (
            Self {
                config: global_config,
                running_checks: match global_config.check().unwrap() {
                    CheckCommands::All(_) => true,
                    CheckCommands::Sanity(_) => false,
                },

                stats_send: stats_send_chan.clone(),
                data_recv_chan: data_recv,
                its_cdp_validator: its::cdp_running::CdpRunningValidator::new(
                    global_config,
                    stats_send_chan,
                ),
                rdh_running_validator: RdhCruRunningChecker::default(),
                rdh_sanity_validator,
                prev_rdhs: ConstGenericRingBuffer::<_, 2>::new(),
            },
            data_send,
        )
    }

    /// Event loop where data is received and validation starts
    pub fn run(&mut self) {
        while let Ok(cdp) = self.data_recv_chan.recv() {
            self.do_checks(cdp);
        }
    }

    fn do_checks(&mut self, cdp_tuple: CdpTuple<T>) {
        let (rdh, payload, rdh_mem_pos) = cdp_tuple;

        self.do_rdh_checks(&rdh, rdh_mem_pos);

        if let Some(system) = self.config.check().unwrap().target() {
            match system {
                System::ITS | System::ITS_Stave => {
                    if !payload.is_empty() {
                        super::its::lib::do_payload_checks(
                            (&rdh, &payload, rdh_mem_pos),
                            &self.stats_send,
                            &mut self.its_cdp_validator,
                        )
                        .unwrap();
                    }
                } // Example of how to add a new system to the validator
                  //
                  // 1. Match on the system target in the config
                  //  config::System::NewSystem => {
                  //     if !payload.is_empty() {
                  // 2. Call the do_payload_checks in the `new_system` module and pass the necessary arguments to do the checks
                  //         super::new_system::lib::do_payload_checks(
                  //             (&rdh, &payload, rdh_mem_pos),
                  //             &self.stats_send_chan,
                  //             &mut self.new_system_cdp_validator,
                  //         );
                  //     }
                  // }
            }
        }

        self.prev_rdhs.push(rdh);
    }

    fn do_rdh_checks(&mut self, rdh: &T, rdh_mem_pos: u64) {
        if let Err(e) = self.rdh_sanity_validator.sanity_check(rdh) {
            self.report_rdh_error(rdh, e, rdh_mem_pos);
        }

        if self.running_checks {
            if let Err(e) = self.rdh_running_validator.check(rdh) {
                self.report_rdh_error(rdh, e, rdh_mem_pos);
            }
        }
    }

    fn report_rdh_error(&mut self, rdh: &T, mut error: String, rdh_mem_pos: u64) {
        // Add additional context unless errors are muted
        if !self.config.mute_errors() {
            error.push('\n');
            error.push_str(RdhCru::rdh_header_text_with_indent_to_string(13).as_str());

            self.prev_rdhs.iter().for_each(|prev_rdh| {
                error.push_str(&format!("  previous:  {prev_rdh}\n"));
            });
            error.push_str(&format!("  current :  {rdh} <--- Error detected here\n"));
        }

        self.stats_send
            .send(StatType::Error(format!("{rdh_mem_pos:#X}: {error}").into()))
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::check::CmdPathArg;
    use alice_protocol_reader::prelude::test_data::CORRECT_RDH_CRU_V7;

    static CFG_TEST_RUN_LINK_VALIDATOR: OnceLock<MockConfig> = OnceLock::new();

    #[test]
    fn test_run_link_validator() {
        let (stats_send_chan, stats_recv_chan) = flume::unbounded();
        let mut mock_config = MockConfig::new();
        mock_config.check = Some(CheckCommands::Sanity(CheckModeArgs {
            target: None,
            path: CmdPathArg::default(),
        }));
        CFG_TEST_RUN_LINK_VALIDATOR.set(mock_config).unwrap();

        let (mut link_validator, _cdp_tuple_send_ch) =
            LinkValidator::new(CFG_TEST_RUN_LINK_VALIDATOR.get().unwrap(), stats_send_chan);

        assert!(!link_validator.running_checks);

        // Spawn the link validator in a thread
        let _handle = thread::spawn(move || {
            link_validator.run();
        });

        // Send a CDP to the link validator
        let cdp = (
            CORRECT_RDH_CRU_V7,
            vec![0x00, 0x01, 0x02],
            0x0000_0000_0000_0000,
        );
        _cdp_tuple_send_ch.send(cdp).unwrap();

        // Wait for the link validator to process the CDP
        thread::sleep(Duration::from_millis(100));

        // Check that the link validator has not sent any errors
        let stats_msg = stats_recv_chan.try_recv();
        assert!(stats_msg.is_err());
    }

    static CFG_TEST_VALID_PAYLOADS_FLAVOR_0: OnceLock<MockConfig> = OnceLock::new();
    #[test]
    fn test_valid_payloads_flavor_0() {
        let mut mock_config = MockConfig::new();
        mock_config.check = Some(CheckCommands::Sanity(CheckModeArgs {
            target: Some(System::ITS),
            ..Default::default()
        }));
        CFG_TEST_VALID_PAYLOADS_FLAVOR_0.set(mock_config).unwrap();

        let (stats_send_chan, stats_recv_chan) = flume::unbounded();

        let (mut link_validator, cdp_tuple_send_ch) = LinkValidator::new(
            CFG_TEST_VALID_PAYLOADS_FLAVOR_0.get().unwrap(),
            stats_send_chan,
        );

        assert!(!link_validator.running_checks);

        // Spawn the link validator in a thread
        let _handle = thread::spawn(move || {
            link_validator.run();
        });

        let mut payload = START_PAYLOAD_FLAVOR_0.to_vec();
        payload.extend_from_slice(&MIDDLE_PAYLOAD_FLAVOR_0);
        payload.extend_from_slice(&END_PAYLOAD_FLAVOR_0);

        // Send a CDP to the link validator
        let cdp = (CORRECT_RDH_CRU_V7, payload, 0);

        cdp_tuple_send_ch.send(cdp).unwrap();

        // Wait for the link validator to process the CDP
        thread::sleep(Duration::from_millis(100));

        // Check that the link validator has not sent any errors
        while let Ok(stats_msg) = stats_recv_chan.try_recv() {
            match stats_msg {
                StatType::Error(_) => panic!("Received error message: {:?}", stats_msg),
                _ => println!("Received stats message: {:?}", stats_msg),
            }
        }
    }

    static CFG_TEST_VALID_PAYLOADS_FLAVOR_2: OnceLock<MockConfig> = OnceLock::new();
    #[test]
    fn test_valid_payloads_flavor_2() {
        let mut mock_config = MockConfig::new();
        mock_config.check = Some(CheckCommands::Sanity(CheckModeArgs {
            target: Some(System::ITS),
            ..Default::default()
        }));
        CFG_TEST_VALID_PAYLOADS_FLAVOR_2.set(mock_config).unwrap();
        let (stats_send_chan, stats_recv_chan) = flume::unbounded();

        let (mut link_validator, cdp_tuple_send_ch) = LinkValidator::new(
            CFG_TEST_VALID_PAYLOADS_FLAVOR_2.get().unwrap(),
            stats_send_chan,
        );

        assert!(!link_validator.running_checks);

        // Spawn the link validator in a thread
        let _handle = thread::spawn(move || {
            link_validator.run();
        });

        let mut payload = START_PAYLOAD_FLAVOR_2.to_vec();
        payload.extend_from_slice(&MIDDLE_PAYLOAD_FLAVOR_2);
        payload.extend_from_slice(&END_PAYLOAD_FLAVOR_2);

        // Send a CDP to the link validator
        let cdp = (CORRECT_RDH_CRU_V7, payload, 0);

        cdp_tuple_send_ch.send(cdp).unwrap();

        // Wait for the link validator to process the CDP
        thread::sleep(Duration::from_millis(100));

        // Check that the link validator has not sent any errors
        while let Ok(stats_msg) = stats_recv_chan.try_recv() {
            match stats_msg {
                StatType::Error(_) => panic!("Received error message: {:?}", stats_msg),
                _ => println!("Received stats message: {:?}", stats_msg),
            }
        }
    }

    static CFG_TEST_INVALID_PAYLOADS_FLAVOR_2_BAD_TDH_ONE_ERROR: OnceLock<MockConfig> =
        OnceLock::new();

    #[test]
    fn test_invalid_payloads_flavor_2_bad_tdh_one_error() {
        let mut mock_config = MockConfig::new();
        mock_config.check = Some(CheckCommands::Sanity(CheckModeArgs {
            target: Some(System::ITS),
            ..Default::default()
        }));
        CFG_TEST_INVALID_PAYLOADS_FLAVOR_2_BAD_TDH_ONE_ERROR
            .set(mock_config)
            .unwrap();
        let (stats_send_chan, stats_recv_chan) = flume::unbounded();
        let (mut link_validator, cdp_tuple_send_ch) = LinkValidator::new(
            CFG_TEST_INVALID_PAYLOADS_FLAVOR_2_BAD_TDH_ONE_ERROR
                .get()
                .unwrap(),
            stats_send_chan,
        );

        assert!(!link_validator.running_checks);

        // Spawn the link validator in a thread
        let _handle = thread::spawn(move || {
            link_validator.run();
        });

        let mut payload = START_PAYLOAD_FLAVOR_2.to_vec();
        payload.extend_from_slice(&MIDDLE_PAYLOAD_FLAVOR_2);
        payload.extend_from_slice(&END_PAYLOAD_FLAVOR_2);
        payload[19] = 0xE9; // Change the TDH to an invalid value

        // Send a CDP to the link validator
        let cdp = (CORRECT_RDH_CRU_V7, payload, 0);

        cdp_tuple_send_ch.send(cdp).unwrap();

        // Wait for the link validator to process the CDP
        thread::sleep(Duration::from_millis(100));

        // Check that the link validator has sent an error
        let stats_msg = stats_recv_chan.try_recv().unwrap();
        match stats_msg {
            StatType::Error(_) => println!("Received error message: {:?}", stats_msg),
            _ => panic!("Received stats message: {:?}", stats_msg),
        }

        // Check that the link validator has not sent any more errors
        while let Ok(stat) = stats_recv_chan.try_recv() {
            match stat {
                StatType::Error(msg) => panic!("Received error message: {msg}"),
                _ => println!("Received stats message: {:?}", stat),
            }
        }
    }

    static CFG_TEST_INIT_LINK_VALIDATOR_NO_CHECKS_ENABLED: OnceLock<MockConfig> = OnceLock::new();

    #[test]
    #[should_panic]
    fn test_init_link_validator_no_checks_enabled() {
        // Should panic because no checks are enabled in the config, doesn't make sense to run the link validator
        let (stats_send_chan, _) = flume::unbounded();

        let mut cfg = MockConfig::new();
        cfg.check = Some(CheckCommands::Sanity(CheckModeArgs::default()));

        type RdhV7 = RdhCru;

        let (mut _link_validator, _cdp_tuple_send_ch): (
            LinkValidator<RdhV7, MockConfig>,
            crossbeam_channel::Sender<CdpTuple<RdhV7>>,
        ) = LinkValidator::new(
            CFG_TEST_INIT_LINK_VALIDATOR_NO_CHECKS_ENABLED
                .get()
                .unwrap(),
            stats_send_chan,
        );
    }
}
