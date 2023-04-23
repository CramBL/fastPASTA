//! Contains the [LinkValidator] that contains all the subvalidators, and delegates all checks for a specific link.
//!
//! A [LinkValidator] is created for each link that is being checked.
//! The [LinkValidator] is responsible for creating and running all the subvalidators.
//! It also contains an [AllocRingBuffer] that is used to store the previous two [RDH]s, to be able to include them in error messages.
use crate::{util::lib::Config, words::lib::RDH};
use ringbuffer::{AllocRingBuffer, RingBufferExt, RingBufferWrite};

struct LinkValidatorConfig {
    running_checks: bool,
    target: Option<crate::util::config::System>,
}

impl LinkValidatorConfig {
    pub fn new(config: &impl crate::util::lib::Checks) -> Self {
        use crate::util::config::Check;
        match config.check() {
            Some(check) => match check {
                Check::All(_) => Self {
                    running_checks: true,
                    target: check.target(),
                },
                _ => Self {
                    running_checks: false,
                    target: check.target(),
                },
            },
            None => Self {
                running_checks: false,
                target: None,
            },
        }
    }
}

/// Main validator that handles all checks on a specific link.
///
/// A [LinkValidator] is created for each link that is being checked.
pub struct LinkValidator<T: RDH> {
    config: LinkValidatorConfig,
    /// Producer channel to send stats through.
    pub send_stats_ch: std::sync::mpsc::Sender<crate::stats::stats_controller::StatType>,
    /// Consumer channel to receive data from.
    pub data_rcv_channel: crossbeam_channel::Receiver<CdpTuple<T>>,
    cdp_validator: crate::validators::cdp_running::CdpRunningValidator<T>,
    rdh_running_validator: crate::validators::rdh_running::RdhCruRunningChecker<T>,
    rdh_sanity_validator: crate::validators::rdh::RdhCruSanityValidator<T>,
    prev_rdhs: AllocRingBuffer<T>,
}

type CdpTuple<T> = (T, Vec<u8>, u64);

impl<T: RDH> LinkValidator<T> {
    /// Creates a new [LinkValidator] and the [StatType][crate::stats::stats_controller::StatType] sender channel to it, from a [Config].
    pub fn new(
        global_config: &impl Config,
        send_stats_ch: std::sync::mpsc::Sender<crate::stats::stats_controller::StatType>,
    ) -> (Self, crossbeam_channel::Sender<CdpTuple<T>>) {
        let local_cfg = LinkValidatorConfig::new(global_config);
        let rdh_sanity_validator = if let Some(system) = local_cfg.target.clone() {
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
                config: local_cfg,
                send_stats_ch: send_stats_ch.clone(),
                data_rcv_channel,
                cdp_validator: crate::validators::cdp_running::CdpRunningValidator::new(
                    global_config,
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

        if let Some(system) = &self.config.target {
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
        if self.config.running_checks {
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
        match preprocess_payload(payload, data_format) {
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

/// Utility function to preprocess the payload and return an iterator over the GBT words
pub fn preprocess_payload(
    payload: &[u8],
    data_format: u8,
) -> Result<impl Iterator<Item = &[u8]>, String> {
    // Retrieve end of payload padding from payload
    let ff_padding = payload
        .iter()
        .rev()
        .take_while(|&x| *x == 0xFF)
        .collect::<Vec<_>>();

    // Exceeds the maximum padding of 15 bytes that is required to pad to 16 bytes
    if ff_padding.len() > 15 {
        return Err(format!("End of payload 0xFF padding is {} bytes, exceeding max of 15 bytes: Skipping current payload",
    ff_padding.len()));
    }

    // Determine if padding is flavor 0 (6 bytes of 0x00 padding following GBT words) or flavor 1 (no padding)
    // Using an iterator approach instead of indexing also supports the case where the payload is smaller than 16 bytes or even empty
    let detected_data_format = if payload
    .iter() // Create an iterator over the payload
    .take(16) // Take the first 16 bytes
    .rev() // Now reverse the iterator
    .take_while(|&x| *x == 0x00) // Take bytes while they are equal to 0x00
    .count() // Count them and check if they are equal to 6
    == 6
    {
        log::trace!("Data format 0 detected");
        0
    } else {
        log::trace!("Data format 2 detected");
        2
    };

    // Split payload into GBT words sized slices, using chunks_exact to allow more compiler optimizations
    let gbt_word_chunks = if detected_data_format == 0 {
        // If flavor 0, dividing into 16 byte chunks should cut the payload up with no remainder
        let chunks = payload.chunks_exact(16);
        debug_assert!(chunks.remainder().is_empty());
        debug_assert!(data_format == 0);
        chunks
    }
    // If flavor 1, and the padding is more than 9 bytes, padding will be processed as a GBT word, therefor exclude it from the slice
    //    Before calling chunks_exact
    else if ff_padding.len() > 9 {
        let last_idx_before_padding = payload.len() - ff_padding.len();
        let chunks = payload[..last_idx_before_padding].chunks_exact(10);
        debug_assert!(chunks.remainder().is_empty());
        debug_assert!(data_format == 2);
        chunks
    } else {
        // Simply divide into 10 byte chunks and assert that the remainder is padding bytes
        let chunks = payload.chunks_exact(10);
        debug_assert!(chunks.remainder().iter().all(|&x| x == 0xFF)); // Asserts that the payload padding is 0xFF
        debug_assert!(data_format == 2);
        chunks
    };

    Ok(gbt_word_chunks)
}
