//! Contains the [LinkValidator] that contains all the [RDH] subvalidators, and delegates all checks for a specific link.
//!
//! A [LinkValidator] is created for each link that is being checked.
//! The [LinkValidator] is responsible for creating and running all the [RDH] subvalidators, and delegating payload depending on target system.
//! It also contains an [AllocRingBuffer] that is used to store the previous two [RDH]s, to be able to include them in error messages.
//!
//! Adding a new system to the validator is done by adding a new module to the [validators](crate::validators) module, and adding the new system to the [System](crate::util::config::System) enum.
//! The new module should contain a main payload validator that can be used by the [LinkValidator] to delegate payload to.
//! Unfortunately it cannot be implemented through trait objects as they cannot be stored in the [LinkValidator] without using dynamic traits.
//!
//! In the `do_checks` function, the [LinkValidator] will delegate the payload to the correct validator depending on the target system.
//! The new system should be added to the match statement, along with how to delegate the payload to the new validator.

pub(crate) use super::{its, rdh, rdh_running::RdhCruRunningChecker};
use crate::{
    stats::lib::StatType,
    util::{
        config::{self, Cfg, Check, System},
        lib::Checks,
    },
    validators::rdh::RdhCruSanityValidator,
    words::{
        lib::RDH,
        rdh_cru::{RdhCRU, V7},
    },
};
use ringbuffer::{AllocRingBuffer, RingBufferExt, RingBufferWrite};

/// Main validator that handles all checks on a specific link.
///
/// A [LinkValidator] is created for each link that is being checked.
pub struct LinkValidator<T: RDH> {
    running_checks: bool,
    /// Producer channel to send stats through.
    pub send_stats_ch: std::sync::mpsc::Sender<StatType>,
    /// Consumer channel to receive data from.
    pub data_rcv_channel: crossbeam_channel::Receiver<CdpTuple<T>>,
    its_cdp_validator: its::cdp_running::CdpRunningValidator<T>,
    rdh_running_validator: RdhCruRunningChecker<T>,
    rdh_sanity_validator: RdhCruSanityValidator<T>,
    prev_rdhs: AllocRingBuffer<T>,
    target_system: Option<System>,
}

type CdpTuple<T> = (T, Vec<u8>, u64);

impl<T: RDH> LinkValidator<T> {
    /// Capacity of the channel (FIFO) to Link Validator threads in terms of CDPs (RDH, Payload, Memory position)
    ///
    /// Larger capacity means less overhead, but more memory usage
    /// Too small capacity will cause the producer thread to block
    const CHANNEL_CDP_CAPACITY: usize = 100; // associated constant

    /// Creates a new [LinkValidator] and the [StatType] sender channel to it, from a [Config].
    pub fn new(
        send_stats_ch: std::sync::mpsc::Sender<StatType>,
    ) -> (Self, crossbeam_channel::Sender<CdpTuple<T>>) {
        let rdh_sanity_validator = if let Some(system) = Cfg::global().check().unwrap().target() {
            match system {
                config::System::ITS => {
                    RdhCruSanityValidator::<T>::with_specialization(rdh::SpecializeChecks::ITS)
                }
            }
        } else {
            RdhCruSanityValidator::default()
        };
        let (send_channel, data_rcv_channel) =
            crossbeam_channel::bounded(Self::CHANNEL_CDP_CAPACITY);
        (
            Self {
                running_checks: match Cfg::global().check().unwrap() {
                    Check::All(_) => true,
                    Check::Sanity(_) => false,
                },
                target_system: Cfg::global().check().unwrap().target(),
                send_stats_ch: send_stats_ch.clone(),
                data_rcv_channel,
                its_cdp_validator: its::cdp_running::CdpRunningValidator::new(send_stats_ch),
                rdh_running_validator: RdhCruRunningChecker::default(),
                rdh_sanity_validator,
                prev_rdhs: AllocRingBuffer::with_capacity(2),
            },
            send_channel,
        )
    }

    /// Event loop where data is received and validation starts
    pub fn run(&mut self) {
        while let Ok(cdp) = self.data_rcv_channel.recv() {
            self.do_checks(cdp);
        }
        log::trace!("LinkValidator: No more data to process, shutting down");
    }

    fn do_checks(&mut self, cdp_tuple: CdpTuple<T>) {
        let (rdh, payload, rdh_mem_pos) = cdp_tuple;

        self.do_rdh_checks(&rdh, rdh_mem_pos);

        if let Some(system) = &self.target_system {
            match system {
                config::System::ITS => {
                    if !payload.is_empty() {
                        super::its::lib::do_payload_checks(
                            (&rdh, &payload, rdh_mem_pos),
                            &self.send_stats_ch,
                            &mut self.its_cdp_validator,
                        );
                    }
                } // Example of how to add a new system to the validator
                  //
                  // 1. Match on the system target in the config
                  //  config::System::NewSystem => {
                  //     if !payload.is_empty() {
                  // 2. Call the do_payload_checks in the `new_system` module and pass the necessary arguments to do the checks
                  //         super::new_system::lib::do_payload_checks(
                  //             (&rdh, &payload, rdh_mem_pos),
                  //             &self.send_stats_ch,
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
        error.push('\n');
        error.push_str(RdhCRU::<V7>::rdh_header_text_with_indent_to_string(13).as_str());
        self.prev_rdhs.iter().for_each(|prev_rdh| {
            error.push_str(&format!("  previous:  {prev_rdh}\n"));
        });
        error.push_str(&format!("  current :  {rdh} <--- Error detected here\n"));

        self.send_stats_ch
            .send(StatType::Error(format!("{rdh_mem_pos:#X}: {error}")))
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::words::its::test_payloads::*;
    use crate::words::rdh_cru::test_data::CORRECT_RDH_CRU_V7;
    use std::sync::mpsc::Sender;

    fn setup_test_link_validator<T: RDH>(
        is_running_checks: bool,
        is_its_payload_running_checks: bool,
        send_stats: Sender<StatType>,
        data_rcv_channel: crossbeam_channel::Receiver<(T, Vec<u8>, u64)>,
    ) -> LinkValidator<T> {
        let mut its_cdp_validator =
            its::cdp_running::CdpRunningValidator::_new_no_cfg(send_stats.clone());
        its_cdp_validator.running_checks = is_its_payload_running_checks;
        LinkValidator {
            running_checks: is_running_checks,
            send_stats_ch: send_stats,
            data_rcv_channel,
            its_cdp_validator,
            rdh_running_validator: RdhCruRunningChecker::default(),
            rdh_sanity_validator: RdhCruSanityValidator::<T>::default(),
            prev_rdhs: AllocRingBuffer::with_capacity(2),
            target_system: None,
        }
    }

    #[test]
    fn test_run_link_validator() {
        let (send_stats_ch, rcv_stats_ch) = std::sync::mpsc::channel();
        let (data_send_channel, data_rcv_channel) =
            crossbeam_channel::bounded(LinkValidator::<RdhCRU<V7>>::CHANNEL_CDP_CAPACITY);
        let mut link_validator: LinkValidator<RdhCRU<V7>> =
            setup_test_link_validator(false, false, send_stats_ch, data_rcv_channel);

        assert_eq!(link_validator.running_checks, false);

        // Spawn the link validator in a thread
        let _handle = std::thread::spawn(move || {
            link_validator.run();
        });

        // Send a CDP to the link validator
        let cdp = (
            CORRECT_RDH_CRU_V7,
            vec![0x00, 0x01, 0x02],
            0x0000_0000_0000_0000,
        );
        data_send_channel.send(cdp).unwrap();

        // Wait for the link validator to process the CDP
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Check that the link validator has not sent any errors
        let stats_msg = rcv_stats_ch.try_recv();
        assert!(stats_msg.is_err());
    }

    #[test]
    fn test_valid_payloads_flavor_0() {
        let (send_stats_ch, rcv_stats_ch) = std::sync::mpsc::channel();
        let (data_send_channel, data_rcv_channel) =
            crossbeam_channel::bounded(LinkValidator::<RdhCRU<V7>>::CHANNEL_CDP_CAPACITY);
        let mut link_validator: LinkValidator<RdhCRU<V7>> =
            setup_test_link_validator(false, false, send_stats_ch, data_rcv_channel);

        assert_eq!(link_validator.running_checks, false);

        // Spawn the link validator in a thread
        let _handle = std::thread::spawn(move || {
            link_validator.run();
        });

        let mut payload = START_PAYLOAD_FLAVOR_0.to_vec();
        payload.extend_from_slice(&MIDDLE_PAYLOAD_FLAVOR_0);
        payload.extend_from_slice(&END_PAYLOAD_FLAVOR_0);

        // Send a CDP to the link validator
        let cdp = (CORRECT_RDH_CRU_V7, payload, 0);
        data_send_channel.send(cdp).unwrap();

        // Wait for the link validator to process the CDP
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Check that the link validator has not sent any errors
        while let Ok(stats_msg) = rcv_stats_ch.try_recv() {
            match stats_msg {
                StatType::Error(_) => panic!("Received error message: {:?}", stats_msg),
                _ => println!("Received stats message: {:?}", stats_msg),
            }
        }
    }

    #[test]
    fn test_valid_payloads_flavor_2() {
        let (send_stats_ch, rcv_stats_ch) = std::sync::mpsc::channel();
        let (data_send_channel, data_rcv_channel) =
            crossbeam_channel::bounded(LinkValidator::<RdhCRU<V7>>::CHANNEL_CDP_CAPACITY);
        let mut link_validator: LinkValidator<RdhCRU<V7>> =
            setup_test_link_validator(false, false, send_stats_ch, data_rcv_channel);

        assert_eq!(link_validator.running_checks, false);

        // Spawn the link validator in a thread
        let _handle = std::thread::spawn(move || {
            link_validator.run();
        });

        let mut payload = START_PAYLOAD_FLAVOR_2.to_vec();
        payload.extend_from_slice(&MIDDLE_PAYLOAD_FLAVOR_2);
        payload.extend_from_slice(&END_PAYLOAD_FLAVOR_2);

        // Send a CDP to the link validator
        let cdp = (CORRECT_RDH_CRU_V7, payload, 0);
        data_send_channel.send(cdp).unwrap();

        // Wait for the link validator to process the CDP
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Check that the link validator has not sent any errors
        while let Ok(stats_msg) = rcv_stats_ch.try_recv() {
            match stats_msg {
                StatType::Error(_) => panic!("Received error message: {:?}", stats_msg),
                _ => println!("Received stats message: {:?}", stats_msg),
            }
        }
    }

    #[test]
    fn test_invalid_payloads_flavor_2_bad_tdh_one_error() {
        let (send_stats_ch, rcv_stats_ch) = std::sync::mpsc::channel();
        let (data_send_channel, data_rcv_channel) =
            crossbeam_channel::bounded(LinkValidator::<RdhCRU<V7>>::CHANNEL_CDP_CAPACITY);
        let mut link_validator: LinkValidator<RdhCRU<V7>> =
            setup_test_link_validator(false, false, send_stats_ch, data_rcv_channel);
        link_validator.target_system = Some(System::ITS);

        assert_eq!(link_validator.running_checks, false);

        // Spawn the link validator in a thread
        let _handle = std::thread::spawn(move || {
            link_validator.run();
        });

        let mut payload = START_PAYLOAD_FLAVOR_2.to_vec();
        payload.extend_from_slice(&MIDDLE_PAYLOAD_FLAVOR_2);
        payload.extend_from_slice(&END_PAYLOAD_FLAVOR_2);
        payload[19] = 0xE9; // Change the TDH to an invalid value

        // Send a CDP to the link validator
        let cdp = (CORRECT_RDH_CRU_V7, payload, 0);
        data_send_channel.send(cdp).unwrap();

        // Wait for the link validator to process the CDP
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Check that the link validator has sent an error
        let stats_msg = rcv_stats_ch.try_recv().unwrap();
        match stats_msg {
            StatType::Error(_) => println!("Received error message: {:?}", stats_msg),
            _ => panic!("Received stats message: {:?}", stats_msg),
        }

        // Check that the link validator has not sent any more errors
        while let Ok(stats_msg) = rcv_stats_ch.try_recv() {
            match stats_msg {
                StatType::Error(_) => panic!("Received error message: {:?}", stats_msg),
                _ => println!("Received stats message: {:?}", stats_msg),
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_init_link_validator_no_config() {
        // Should panic because config is not set, doesn't make sense to run the link validator
        let (send_stats_ch, _) = std::sync::mpsc::channel();

        let (mut _link_validator, _cdp_tuple_send_ch): (
            LinkValidator<RdhCRU<V7>>,
            crossbeam_channel::Sender<CdpTuple<RdhCRU<V7>>>,
        ) = LinkValidator::new(send_stats_ch);
    }
}
