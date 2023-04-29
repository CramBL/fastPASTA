//! Contains the [LinkValidator] that contains all the subvalidators, and delegates all checks for a specific link.
//!
//! A [LinkValidator] is created for each link that is being checked.
//! The [LinkValidator] is responsible for creating and running all the subvalidators.
//! It also contains an [AllocRingBuffer] that is used to store the previous two [RDH]s, to be able to include them in error messages.

use crate::{util::config::Check, words::lib::RDH};
use ringbuffer::{AllocRingBuffer, RingBufferExt, RingBufferWrite};

/// Main validator that handles all checks on a specific link.
///
/// A [LinkValidator] is created for each link that is being checked.
pub struct LinkValidator<T: RDH, C: crate::util::lib::Config> {
    config: std::sync::Arc<C>,
    running_checks: bool,
    /// Producer channel to send stats through.
    pub send_stats_ch: std::sync::mpsc::Sender<crate::stats::stats_controller::StatType>,
    /// Consumer channel to receive data from.
    pub data_rcv_channel: crossbeam_channel::Receiver<CdpTuple<T>>,
    cdp_validator: crate::validators::cdp_running::CdpRunningValidator<T, C>,
    rdh_running_validator: crate::validators::rdh_running::RdhCruRunningChecker<T>,
    rdh_sanity_validator: crate::validators::rdh::RdhCruSanityValidator<T>,
    prev_rdhs: AllocRingBuffer<T>,
}

type CdpTuple<T> = (T, Vec<u8>, u64);

impl<T: RDH, C: crate::util::lib::Config> LinkValidator<T, C> {
    /// Creates a new [LinkValidator] and the [StatType][crate::stats::stats_controller::StatType] sender channel to it, from a [Config].
    pub fn new(
        global_config: std::sync::Arc<C>,
        send_stats_ch: std::sync::mpsc::Sender<crate::stats::stats_controller::StatType>,
    ) -> (Self, crossbeam_channel::Sender<CdpTuple<T>>) {
        let rdh_sanity_validator = if let Some(system) = global_config.check().unwrap().target() {
            match system {
                crate::util::config::System::ITS => {
                    crate::validators::rdh::RdhCruSanityValidator::<T>::with_specialization(
                        super::rdh::SpecializeChecks::ITS,
                    )
                }
            }
        } else {
            crate::validators::rdh::RdhCruSanityValidator::default()
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
                cdp_validator: crate::validators::cdp_running::CdpRunningValidator::new(
                    global_config.clone(),
                    send_stats_ch,
                ),
                rdh_running_validator:
                    crate::validators::rdh_running::RdhCruRunningChecker::default(),
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
                crate::util::config::System::ITS => {
                    self.cdp_validator.set_current_rdh(&rdh, rdh_mem_pos);
                    if !payload.is_empty() {
                        self.do_payload_checks(&payload, rdh.data_format());
                    }
                }
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
        error.push_str(crate::words::rdh_cru::RdhCRU::<crate::words::rdh_cru::V7>::rdh_header_text_with_indent_to_string(13).as_str());
        self.prev_rdhs.iter().for_each(|prev_rdh| {
            error.push_str(&format!("  previous:  {prev_rdh}\n"));
        });
        error.push_str(&format!("  current :  {rdh} <--- Error detected here\n"));

        self.send_stats_ch
            .send(crate::stats::stats_controller::StatType::Error(format!(
                "{rdh_mem_pos:#X}: {error}"
            )))
            .unwrap();
    }

    fn do_payload_checks(&mut self, payload: &[u8], data_format: u8) {
        match super::lib::preprocess_payload(payload, data_format) {
            Ok(gbt_word_chunks) => gbt_word_chunks.for_each(|gbt_word| {
                self.cdp_validator.check(&gbt_word[..10]); // Take 10 bytes as flavor 0 would have additional 6 bytes of padding
            }),
            Err(e) => {
                self.send_stats_ch
                    .send(crate::stats::stats_controller::StatType::Error(e))
                    .unwrap();
                self.cdp_validator.reset_fsm();
            }
        }
    }
}
