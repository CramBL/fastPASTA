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
        config::{self, Check},
        lib::Config,
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
pub struct LinkValidator<T: RDH, C: Config> {
    config: std::sync::Arc<C>,
    running_checks: bool,
    /// Producer channel to send stats through.
    pub send_stats_ch: std::sync::mpsc::Sender<StatType>,
    /// Consumer channel to receive data from.
    pub data_rcv_channel: crossbeam_channel::Receiver<CdpTuple<T>>,
    its_cdp_validator: its::cdp_running::CdpRunningValidator<T, C>,
    rdh_running_validator: RdhCruRunningChecker<T>,
    rdh_sanity_validator: RdhCruSanityValidator<T>,
    prev_rdhs: AllocRingBuffer<T>,
}

type CdpTuple<T> = (T, Vec<u8>, u64);

impl<T: RDH, C: Config> LinkValidator<T, C> {
    /// Creates a new [LinkValidator] and the [StatType] sender channel to it, from a [Config].
    pub fn new(
        global_config: std::sync::Arc<C>,
        send_stats_ch: std::sync::mpsc::Sender<StatType>,
    ) -> (Self, crossbeam_channel::Sender<CdpTuple<T>>) {
        let rdh_sanity_validator = if let Some(system) = global_config.check().unwrap().target() {
            match system {
                config::System::ITS => {
                    RdhCruSanityValidator::<T>::with_specialization(rdh::SpecializeChecks::ITS)
                }
            }
        } else {
            RdhCruSanityValidator::default()
        };
        let (send_channel, data_rcv_channel) =
            crossbeam_channel::bounded(crate::CHANNEL_CDP_CAPACITY);
        (
            Self {
                config: global_config.clone(),
                running_checks: match global_config.check().unwrap() {
                    Check::All(_) => true,
                    Check::Sanity(_) => false,
                },

                send_stats_ch: send_stats_ch.clone(),
                data_rcv_channel,
                its_cdp_validator: its::cdp_running::CdpRunningValidator::new(
                    global_config.clone(),
                    send_stats_ch,
                ),
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

        if let Some(system) = self.config.check().unwrap().target() {
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
