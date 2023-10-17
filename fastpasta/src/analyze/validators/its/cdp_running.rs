//! Checks the CDP payload. Uses the [ItsPayloadFsmContinuous] state machine to determine which words to expect.
//!
//! [CdpRunningValidator] delegates sanity checks to word specific sanity checkers.

mod readout_frame;

use self::readout_frame::ItsReadoutFrameValidator;

use super::{
    data_words::DATA_WORD_SANITY_CHECKER,
    its_payload_fsm_cont::{self, ItsPayloadFsmContinuous},
    lib::ItsPayloadWord,
    status_word::tdh::TdhValidator,
    util::StatusWordContainer,
};
use crate::config::prelude::*;
use crate::stats::StatType;
use crate::words::its::data_words::ob_data_word_id_to_input_number_connector;
use crate::words::its::data_words::ob_data_word_id_to_lane;
use crate::words::its::status_words::util::is_lane_active;
use crate::words::its::status_words::{
    cdw::Cdw, ddw::Ddw0, ihw::Ihw, tdh::Tdh, tdt::Tdt, StatusWord,
};
use crate::words::its::Stave;
use alice_protocol_reader::prelude::FilterOpt;
use alice_protocol_reader::prelude::RDH;

#[derive(Debug, Clone, Copy)]
enum StatusWordKind<'a> {
    Ihw(&'a [u8]),
    Tdh(&'a [u8]),
    Tdt(&'a [u8]),
    Ddw0(&'a [u8]),
}

/// Checks the CDP payload and reports any errors.
pub struct CdpRunningValidator<T: RDH, C: ChecksOpt + FilterOpt + CustomChecksOpt + 'static> {
    config: &'static C,
    running_checks_enabled: bool,
    its_state_machine: ItsPayloadFsmContinuous,
    status_words: StatusWordContainer,
    current_rdh: Option<T>,
    gbt_word_counter: u16,
    pub(crate) stats_send_ch: flume::Sender<StatType>,
    payload_mem_pos: u64,
    gbt_word_padding_size_bytes: u8,
    is_new_data: bool, // Flag used to indicate start of new CDP payload where a CDW is valid
    // Stores the ALPIDE data from an ITS readout frame, if the config is set to check ALPIDE data, and a filter for a stave is set.
    readout_frame_validator: Option<ItsReadoutFrameValidator<C>>,
}

impl<T: RDH, C: ChecksOpt + FilterOpt + CustomChecksOpt> CdpRunningValidator<T, C> {
    /// Creates a new [CdpRunningValidator] from a config that implements [ChecksOpt] + [FilterOpt] and a [StatType] producer channel.
    pub fn new(config: &'static C, stats_send_ch: flume::Sender<StatType>) -> Self {
        Self {
            config,
            running_checks_enabled: matches!(
                config.check(),
                Some(CheckCommands::All { system: _ })
            ),
            its_state_machine: ItsPayloadFsmContinuous::default(),
            status_words: StatusWordContainer::new_const(),
            current_rdh: None,
            gbt_word_counter: 0,
            stats_send_ch,
            payload_mem_pos: 0,
            gbt_word_padding_size_bytes: 0,
            is_new_data: false,
            readout_frame_validator: if config.check().is_some_and(|check| {
                check
                    .target()
                    .is_some_and(|target| target == System::ITS_Stave)
            }) {
                Some(ItsReadoutFrameValidator::new(config))
            } else {
                None
            },
        }
    }

    /// Helper function to format and report an error
    ///
    /// Takes in the error string slice and the word slice
    /// Adds the current memory position to the error string
    /// Sends the error to the stats channel
    #[inline]
    fn report_error(&self, error: &str, word_slice: &[u8]) {
        super::util::report_error(
            self.calc_current_word_mem_pos(),
            error,
            word_slice,
            &self.stats_send_ch,
        );
    }

    /// Resets the state machine to the initial state and logs a warning
    ///
    /// Use this if a payload format is invalid and the next payload can be processed from the initial state
    #[inline]
    pub fn reset_fsm(&mut self) {
        log::warn!("Resetting CDP Payload FSM");
        self.its_state_machine.reset_fsm();
    }

    /// This function has to be called for every RDH
    ///
    /// It defines what is valid, and is necessary to keep track of the memory position of each word
    /// It uses the RDH to determine size of padding
    #[inline]
    pub fn set_current_rdh(&mut self, rdh: &T, rdh_mem_pos: u64) {
        self.current_rdh = Some(T::load(&mut rdh.to_byte_slice()).unwrap());
        self.payload_mem_pos = rdh_mem_pos + 64;
        if rdh.data_format() == 0 {
            self.gbt_word_padding_size_bytes = 6; // Data format 0
        } else {
            self.gbt_word_padding_size_bytes = 0; // Data format 2
        }
        self.is_new_data = true;
        self.gbt_word_counter = 0;

        // If the an ItsReadoutFrameValidator is present
        // and the stave the data is from is not known yet, then set the stave.
        if self
            .readout_frame_validator
            .as_ref()
            .is_some_and(|rfv| rfv.stave().is_none())
        {
            self.readout_frame_validator
                .as_mut()
                .unwrap()
                .set_stave(Stave::from_feeid(
                    self.current_rdh.as_ref().unwrap().fee_id(),
                ));
        }
    }

    /// This function has to be called for every GBT word
    #[inline]
    pub fn check(&mut self, gbt_word: &[u8]) {
        debug_assert!(gbt_word.len() == 10);
        self.gbt_word_counter += 1; // Tracks the number of GBT words seen in the current CDP

        let current_word = self.its_state_machine.advance(gbt_word);

        // Match the result of the FSM trying to determine the word
        // If the ID is not recognized as valid, the FSM takes a best guess among the
        // valid words in the current state and returns it as an error, that is handled below
        match current_word {
            Ok(word) => match word {
                // DataWord and CDW are handled together
                ItsPayloadWord::DataWord | ItsPayloadWord::CDW => {
                    self.preprocess_data_word(gbt_word)
                }
                ItsPayloadWord::TDH => {
                    self.preprocess_status_word(StatusWordKind::Tdh(gbt_word));
                    if self.running_checks_enabled {
                        self.check_tdh_no_continuation(gbt_word);
                        self.check_tdh_trigger_interval(gbt_word);
                    }
                }
                ItsPayloadWord::TDT => self.preprocess_status_word(StatusWordKind::Tdt(gbt_word)),
                ItsPayloadWord::IHW => {
                    self.preprocess_status_word(StatusWordKind::Ihw(gbt_word));
                    if self.running_checks_enabled {
                        self.check_rdh_at_initial_ihw(gbt_word);
                    }
                }

                ItsPayloadWord::TDH_after_packet_done => {
                    self.preprocess_status_word(StatusWordKind::Tdh(gbt_word));
                    if self.running_checks_enabled {
                        self.check_tdh_by_was_tdt_packet_done_true(gbt_word);
                        self.check_tdh_trigger_interval(gbt_word);
                    }
                }

                ItsPayloadWord::DDW0 => self.preprocess_status_word(StatusWordKind::Ddw0(gbt_word)),

                ItsPayloadWord::TDH_continuation => {
                    self.preprocess_status_word(StatusWordKind::Tdh(gbt_word));
                    if self.running_checks_enabled {
                        self.check_tdh_continuation(gbt_word);
                    }
                }
                ItsPayloadWord::IHW_continuation => {
                    self.preprocess_status_word(StatusWordKind::Ihw(gbt_word))
                }
            },

            Err(ambigious_word) => match ambigious_word {
                its_payload_fsm_cont::AmbigiousError::TDH_or_DDW0 => {
                    self.report_error(
                    "[E990] Unrecognized ID in ITS payload, could be TDH/DDW0 based on current state, attempting to parse as TDH",
                    gbt_word,
                );
                    self.preprocess_status_word(StatusWordKind::Tdh(gbt_word));
                }
                its_payload_fsm_cont::AmbigiousError::DW_or_TDT_CDW => {
                    self.report_error("[E991] Unrecognized ID in ITS payload, could be Data Word/TDT/CDW based on current state, attempting to parse as Data Word", gbt_word);
                    self.preprocess_data_word(gbt_word);
                }
                its_payload_fsm_cont::AmbigiousError::DDW0_or_TDH_IHW => {
                    self.report_error("[E992] Unrecognized ID in ITS payload, could be DDW0/TDH/IHW based on current state, attempting to parse as DDW0", gbt_word);
                    self.preprocess_status_word(StatusWordKind::Ddw0(gbt_word));
                }
            },
        }
    }

    /// Calculates the current position in the memory of the current word.
    ///
    /// Current payload position is the first byte after the current RDH
    /// The gbt word position then relative to the current payload is then:
    /// relative_mem_pos = gbt_word_counter * (10 + gbt_word_padding_size_bytes)
    /// And the absolute position in the memory is then:
    /// gbt_word_mem_pos = payload_mem_pos + relative_mem_pos
    #[inline]
    fn calc_current_word_mem_pos(&self) -> u64 {
        let gbt_word_memory_size_bytes: u64 = 10 + self.gbt_word_padding_size_bytes as u64;
        let relative_mem_pos = (self.gbt_word_counter - 1) as u64 * gbt_word_memory_size_bytes;
        relative_mem_pos + self.payload_mem_pos
    }

    /// Takes a slice of bytes wrapped in an enum of the expected status word then:
    /// 1. Deserializes the slice as the expected status word and checks it for sanity.
    /// 2. If the sanity check fails, the error is sent to the stats channel
    /// 3. Stores the deserialized status word as the last status word of the same type.
    /// 4. Sets flags if appropriate
    #[inline]
    fn preprocess_status_word(&mut self, status_word: StatusWordKind) {
        match status_word {
            StatusWordKind::Tdh(tdh_as_slice) => self.preprocess_tdh(tdh_as_slice),
            StatusWordKind::Tdt(tdt_as_slice) => self.preprocess_tdt(tdt_as_slice),
            StatusWordKind::Ihw(ihw_as_slice) => self.preprocess_ihw(ihw_as_slice),
            StatusWordKind::Ddw0(ddw0_as_slice) => {
                self.preprocess_ddw0(ddw0_as_slice);
            }
        }
    }

    fn preprocess_tdh(&mut self, tdh_slice: &[u8]) {
        let tdh = Tdh::load(&mut <&[u8]>::clone(&tdh_slice)).unwrap();
        if let Err(e) = self.status_words.sanity_check_tdh(&tdh) {
            self.report_error(&format!("[E40] {e}"), tdh_slice);
        }

        self.status_words.replace_tdh(tdh);

        // If the current TDH does not have continuation set, then it is the start of a new readout frame
        if self
            .readout_frame_validator
            .as_ref()
            .is_some_and(|rvf| !rvf.is_in_frame())
            && self.status_words.tdh().unwrap().continuation() == 0
        {
            let start_mem_pos = self.calc_current_word_mem_pos();
            self.readout_frame_validator
                .as_mut()
                .unwrap()
                .new_frame(start_mem_pos);
        }
    }

    fn preprocess_tdt(&mut self, tdh_slice: &[u8]) {
        let tdt = Tdt::load(&mut <&[u8]>::clone(&tdh_slice)).unwrap();
        if let Err(e) = self.status_words.sanity_check_tdt(&tdt) {
            self.report_error(&format!("[E50] {e}"), tdh_slice);
        }
        // Replace TDT before processing ALPIDE readout frame
        self.status_words.replace_tdt(tdt);

        if self.readout_frame_validator.is_some() && self.status_words.tdt().unwrap().packet_done()
        {
            self.process_readout_frame();
        }
    }

    fn preprocess_ihw(&mut self, ihw_slice: &[u8]) {
        let ihw = Ihw::load(&mut <&[u8]>::clone(&ihw_slice)).unwrap();
        if let Err(e) = self.status_words.sanity_check_ihw(&ihw) {
            self.report_error(&format!("[E30] {e}"), ihw_slice);
        }
        self.status_words.replace_ihw(ihw);
    }

    fn preprocess_ddw0(&mut self, ddw0_slice: &[u8]) {
        let ddw0 = Ddw0::load(&mut <&[u8]>::clone(&ddw0_slice)).unwrap();
        if let Err(e) = self.status_words.sanity_check_ddw0(&ddw0) {
            self.report_error(&format!("[E60] {e}"), ddw0_slice);
        }

        // Additional state dependent checks on RDH
        if self.running_checks_enabled {
            self.check_rdh_at_ddw0(ddw0_slice);
        }
        self.status_words.replace_ddw(ddw0);
    }

    /// Takes a slice of bytes expected to be a data word, and checks if it has a valid identifier.
    #[inline]
    fn preprocess_data_word(&mut self, data_word_slice: &[u8]) {
        const ID_INDEX: usize = 9;
        if self.is_new_data && data_word_slice[ID_INDEX] == 0xF8 {
            // CDW
            self.process_cdw(data_word_slice);
        } else {
            // Regular data word
            if let Err(e) = DATA_WORD_SANITY_CHECKER.check_any(data_word_slice) {
                self.report_error(&format!("[E70] {e}"), data_word_slice);
            }

            let id_3_msb = data_word_slice[ID_INDEX] >> 5;
            if id_3_msb == 0b001 {
                // Inner Barrel
                self.process_ib_data_word(data_word_slice);
            } else if id_3_msb == 0b010 {
                // Outer Barrel
                self.process_ob_data_word(data_word_slice);
            }
        }

        self.is_new_data = false;
    }

    #[inline]
    fn process_ib_data_word(&mut self, ib_slice: &[u8]) {
        if !self.running_checks_enabled {
            return;
        }
        let lane_id = ib_slice[9] & 0x1F;
        // lane in active_lanes
        let active_lanes = self.status_words.ihw().unwrap().active_lanes();
        if !is_lane_active(lane_id, active_lanes) {
            self.report_error(
                &format!("[E72] IB lane {lane_id} is not active according to IHW active_lanes: {active_lanes:#X}."),
                ib_slice,
            );
        }
        // Matches if there is an ITS readout frame validator.
        // If not we are not collecting data ie. ALPIDE checks are not enabled.
        if let Some(frame_validator) = &mut self.readout_frame_validator {
            frame_validator.store_lane_data(ib_slice);
        }
    }

    #[inline]
    fn process_ob_data_word(&mut self, ob_slice: &[u8]) {
        if !self.running_checks_enabled {
            return;
        }

        let lane_id = ob_data_word_id_to_lane(ob_slice[9]);
        // lane in active_lanes
        let active_lanes = self.status_words.ihw().unwrap().active_lanes();
        if !is_lane_active(lane_id, active_lanes) {
            self.report_error(
                &format!("[E71] OB lane {lane_id} is not active according to IHW active_lanes: {active_lanes:#X}."),
                ob_slice,
            );
        }

        // lane in connector <= 6
        let input_number_connector = ob_data_word_id_to_input_number_connector(ob_slice[9]);
        if input_number_connector > 6 {
            self.report_error(
                &format!("[E73] OB Data Word has input connector {input_number_connector} > 6."),
                ob_slice,
            );
        }
        // If there is no readout frame, we are not collecting data.
        if let Some(rvf) = self.readout_frame_validator.as_mut() {
            rvf.store_lane_data(ob_slice);
        }
    }

    #[inline]
    fn process_cdw(&mut self, cdw_slice: &[u8]) {
        if !self.running_checks_enabled {
            return;
        }
        let cdw = Cdw::load(&mut <&[u8]>::clone(&cdw_slice)).unwrap();

        // If this is not the first CDW, check that the user fields matches the previous CDW
        if self.status_words.cdw().is_some_and(|prv_cdw| {
            prv_cdw.calibration_user_fields() != cdw.calibration_user_fields()
                && cdw.calibration_word_index() != 0
        }) {
            self.report_error("[E81] CDW index is not 0", cdw_slice);
        }

        self.status_words.replace_cdw(cdw);
    }

    // Minor checks done in certain states

    /// Checks TDH trigger and continuation following a TDT packet_done = 1
    #[inline]
    fn check_tdh_by_was_tdt_packet_done_true(&mut self, tdh_slice: &[u8]) {
        if TdhValidator::check_after_tdt_packet_done_true(&self.status_words).is_err() {
            self.report_error(
                &format!(
                    "[E440] TDH trigger_bc is not increasing, previous: {:#X}, current: {:#X}.",
                    self.status_words.prv_tdh().unwrap().trigger_bc(),
                    self.status_words.tdh().unwrap().trigger_bc()
                ),
                tdh_slice,
            );
        }
    }

    /// Checks RDH stop_bit and pages_counter when a DDW0 is observed
    #[inline]
    fn check_rdh_at_ddw0(&mut self, ddw0_slice: &[u8]) {
        if self.current_rdh.as_ref().unwrap().stop_bit() != 1 {
            self.report_error("[E110] DDW0 observed but RDH stop bit is not 1", ddw0_slice);
        }
        if self.current_rdh.as_ref().unwrap().pages_counter() == 0 {
            self.report_error("[E111] DDW0 observed but RDH page counter is 0", ddw0_slice);
        }
    }
    /// Checks RDH stop_bit and pages_counter when an initial IHW is observed (not IHW during continuation)
    #[inline]
    fn check_rdh_at_initial_ihw(&mut self, ihw_slice: &[u8]) {
        if self.current_rdh.as_ref().unwrap().stop_bit() != 0 {
            self.report_error("[E12] IHW observed but RDH stop bit is not 0", ihw_slice);
        }
    }

    /// Checks TDH when continuation is expected (Previous TDT packet_done = 0)
    #[inline]
    fn check_tdh_continuation(&mut self, tdh_slice: &[u8]) {
        if self.status_words.tdh().unwrap().continuation() != 1 {
            self.report_error("[E41] TDH continuation is not 1", tdh_slice);
        }

        if let Some(previous_tdh) = self.status_words.prv_tdh() {
            if previous_tdh.trigger_bc() != self.status_words.tdh().unwrap().trigger_bc() {
                self.report_error("[E441] TDH trigger_bc is not the same", tdh_slice);
            }
            if previous_tdh.trigger_orbit() != self.status_words.tdh().unwrap().trigger_orbit() {
                self.report_error("[E442] TDH trigger_orbit is not the same", tdh_slice);
            }
            if previous_tdh.trigger_type() != self.status_words.tdh().unwrap().trigger_type() {
                self.report_error("[E443] TDH trigger_type is not the same", tdh_slice);
            }
        }
    }

    /// Checks TDH fields: continuation, orbit, when the TDH immediately follows an IHW
    #[inline]
    fn check_tdh_no_continuation(&mut self, tdh_slice: &[u8]) {
        // 1. let bindings to RDH and TDH
        let current_rdh = self.current_rdh.as_ref().expect("RDH should be set");
        let current_tdh = self
            .status_words
            .tdh()
            .expect("TDH should be set, process words before checks");

        if let Err(errs) = TdhValidator::check_tdh_no_continuation(current_tdh, current_rdh) {
            errs.into_iter().for_each(|err| {
                self.report_error(&err, tdh_slice);
            })
        }
    }

    /// Checks if the TDH trigger_bc period matches the specified value
    ///
    /// reports an error with the detected erroneous period if the check fails
    ///
    /// The check is only applicable to consecutive TDHs with internal_trigger set.
    fn check_tdh_trigger_interval(&self, _tdh_slice: &[u8]) {
        if let Some(specified_trig_period) = self.config.check_its_trigger_period() {
            if let Some(prev_int_tdh) = self.status_words.tdh_previous_with_internal_trg() {
                let current_tdh = self
                    .status_words
                    .tdh()
                    .expect("TDH should be set, process words before checks");

                if current_tdh.internal_trigger() == 1 {
                    if let Err(err_msg) = TdhValidator::check_trigger_interval(
                        current_tdh,
                        prev_int_tdh,
                        specified_trig_period,
                    ) {
                        self.stats_send_ch
                            .send(StatType::Error(
                                format!(
                                    "{mem_pos:#X}: {err_msg} ",
                                    mem_pos = self.calc_current_word_mem_pos()
                                )
                                .into(),
                            ))
                            .expect("Failed to send error to stats channel")
                    }
                }
            }
        }
    }

    /// Close a readout frame by supplying the current memory position
    ///
    /// And start the processing by the [ItsReadoutFrameValidator]
    fn process_readout_frame(&mut self) {
        let frame_end_pos = self.calc_current_word_mem_pos();
        if self
            .readout_frame_validator
            .as_mut()
            .unwrap()
            .try_close_frame(frame_end_pos)
            .is_ok()
        {
            self.readout_frame_validator
                .as_mut()
                .unwrap()
                .process_frame(
                    &self.stats_send_ch,
                    &self.status_words,
                    self.current_rdh.as_ref().unwrap(),
                );
        } else {
            let err_msg = format!("{mem_pos:#X}: [E59] TDT with packet done marked the end of a readout frame, but a start of readout frame was never seen (TDH with continuation = 0)", mem_pos = self.calc_current_word_mem_pos());
            self.stats_send_ch
                .send(StatType::Error(err_msg.into()))
                .unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_util::MockConfig;
    use alice_protocol_reader::{
        prelude::{test_data::CORRECT_RDH_CRU_V7, RdhCru},
        rdh::{test_data::CORRECT_RDH_CRU_V7_SOT, RDH_CRU},
    };
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
    use std::sync::OnceLock;

    static MOCK_CONFIG_DEFAULT: OnceLock<MockConfig> = OnceLock::new();
    fn get_default_config() -> &'static MockConfig {
        MOCK_CONFIG_DEFAULT.get_or_init(MockConfig::default)
    }
    static MOCK_CONFIG_RUNNING_CHECKS: OnceLock<MockConfig> = OnceLock::new();
    fn get_running_checks_config() -> &'static MockConfig {
        MOCK_CONFIG_RUNNING_CHECKS.get_or_init(MockConfig::new_check_all_its)
    }

    #[test]
    fn test_validate_ihw() {
        const VALID_ID: u8 = 0xE0;
        const _ACTIVE_LANES_14_ACTIVE: u32 = 0x3F_FF;
        let raw_data_ihw = [
            0xFF, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, VALID_ID,
        ];
        let (send, stats_recv_ch) = flume::unbounded();

        let mut validator: CdpRunningValidator<RdhCru, MockConfig> =
            CdpRunningValidator::new(get_default_config(), send);
        let rdh_mem_pos = 0;

        validator.set_current_rdh(&CORRECT_RDH_CRU_V7, rdh_mem_pos);
        validator.check(&raw_data_ihw);

        assert!(stats_recv_ch.try_recv().is_err()); // Checks that no error was received (nothing received)
    }

    #[test]
    fn test_invalidate_ihw() {
        const INVALID_ID: u8 = 0xE1;
        const _ACTIVE_LANES_14_ACTIVE: u32 = 0x3F_FF;
        let raw_data_ihw = [
            0xFF, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, INVALID_ID,
        ];

        let (send, stats_recv_ch) = flume::unbounded();
        let mut validator: CdpRunningValidator<RdhCru, MockConfig> =
            CdpRunningValidator::new(get_default_config(), send);
        let rdh_mem_pos = 0x0;

        validator.set_current_rdh(&CORRECT_RDH_CRU_V7, rdh_mem_pos);
        validator.check(&raw_data_ihw);

        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    &*msg,
                    "0x40: [E30] ID is not 0xE0: 0xE1  [FF 3F 00 00 00 00 00 00 00 E1]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
        // No more errors
        assert!(stats_recv_ch.try_recv().is_err());
    }

    #[test]
    fn test_expect_ihw_invalidate_tdh() {
        const _VALID_ID: u8 = 0xF0;
        // Boring but very typical TDT, everything is 0 except for packet_done
        let raw_data_tdt = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF1];

        let (send, stats_recv_ch) = flume::unbounded();
        let mut validator: CdpRunningValidator<RdhCru, MockConfig> =
            CdpRunningValidator::new(get_default_config(), send);
        let rdh_mem_pos = 0x0; // RDH size is 64 bytes

        validator.set_current_rdh(&CORRECT_RDH_CRU_V7, rdh_mem_pos); // Data format is 2
        validator.check(&raw_data_tdt);

        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    &*msg,
                    "0x40: [E30] ID is not 0xE0: 0xF1  [00 00 00 00 00 00 00 00 01 F1]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_tdh_trigger_bc_increasing_fail() {
        // ARRANGE
        // RDH -> IHW -> TDH0 no_data -> TDH1
        let raw_data_ihw = [0xFF, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0];
        let raw_data_tdh0 = [0x03, 0x3A, 0x01, 0x00, 0x75, 0xD5, 0x7D, 0x0B, 0x00, 0xE8];
        let tdh0 = Tdh::load(&mut raw_data_tdh0.as_slice()).unwrap();
        println!("cont:{}", tdh0.continuation());
        println!("int:{}", tdh0.internal_trigger());
        println!("no_data={}", tdh0.no_data());
        assert_eq!(tdh0.no_data(), 1);
        let raw_data_tdh1 = [0x03, 0x1A, 0x00, 0x00, 0x75, 0xD5, 0x7D, 0x0B, 0x00, 0xE8];
        let tdh1 = Tdh::load(&mut raw_data_tdh1.as_slice()).unwrap();
        // They are TDH0 is larger than TDH1 which is an error.
        assert!(tdh0.trigger_bc() > tdh1.trigger_bc());

        let (send, stats_recv_ch) = flume::unbounded();
        let mut validator: CdpRunningValidator<RdhCru, MockConfig> =
            CdpRunningValidator::new(get_running_checks_config(), send);

        // ACT
        validator.set_current_rdh(&CORRECT_RDH_CRU_V7, 0);
        validator.check(&raw_data_ihw);
        validator.check(&raw_data_tdh0);
        validator.check(&raw_data_tdh1);

        // ASSERT (receive message and assert it is expected)
        // First we get an error that the first TDH trigger_bc doesn't match the RDH bc
        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => assert_str_eq!("0x4A: [E445] TDH trigger_bc is not equal to RDH bc, TDH: 0x1, RDH: 0x0. [03 3A 01 00 75 D5 7D 0B 00 E8]", &*msg),
            _ => unreachable!(),
        }
        // Then we get the TDH trigger_bc mismatch
        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => assert_str_eq!("0x54: [E440] TDH trigger_bc is not increasing, previous: 0x1, current: 0x0. [03 1A 00 00 75 D5 7D 0B 00 E8]", &*msg),
            _ => unreachable!(),
        }
        // No more errors
        assert!(stats_recv_ch.try_recv().is_err());
    }

    #[test]
    fn test_expect_match_rdh_tdh_trigger_type_fail() {
        // ARRANGE
        const TDH_TRIGGER_TYPE: u16 = 0xA03;
        let rdh_trig_type_12_lsb = CORRECT_RDH_CRU_V7_SOT.rdh2().trigger_type as u16 & 0xFFF;
        assert_ne!(rdh_trig_type_12_lsb, TDH_TRIGGER_TYPE);
        let raw_data_ihw = [0xFF, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0];
        let raw_data_tdh = [0x03, 0x1A, 0x00, 0x00, 0x75, 0xD5, 0x7D, 0x0B, 0x00, 0xE8];
        let tdh = Tdh::load(&mut raw_data_tdh.as_slice()).unwrap();
        assert_eq!(tdh.trigger_type(), TDH_TRIGGER_TYPE);
        assert_eq!(tdh.internal_trigger(), 1);

        let (send, stats_recv_ch) = flume::unbounded();
        let mut validator: CdpRunningValidator<RdhCru, MockConfig> =
            CdpRunningValidator::new(get_running_checks_config(), send);

        // The check is only triggered by an RDH with page counter 0 and pht trigger
        assert_eq!(CORRECT_RDH_CRU_V7_SOT.pages_counter(), 0);
        assert!(CORRECT_RDH_CRU_V7_SOT.rdh2().is_pht_trigger());

        // ACT
        validator.set_current_rdh(&CORRECT_RDH_CRU_V7_SOT, 0);
        validator.check(&raw_data_ihw);
        validator.check(&raw_data_tdh);

        // ASSERT (receive message and assert it is expected)
        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_str_eq!(&*msg, "0x4A: [E44] TDH trigger_type 0xA03 != 0x893 RDH trigger_type[11:0]. [03 1A 00 00 75 D5 7D 0B 00 E8]");
            }
            _ => unreachable!(),
        }
        // No more errors
        assert!(stats_recv_ch.try_recv().is_err());
    }

    #[test]
    fn test_expect_ihw_invalidate_tdh_and_next() {
        const _VALID_ID: u8 = 0xF0;
        // Boring but very typical TDT, everything is 0 except for packet_done
        let raw_data_tdt = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF1];
        let raw_data_tdt_next = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF2];

        let (send, stats_recv_ch) = flume::unbounded();
        let mut validator: CdpRunningValidator<RdhCru, MockConfig> =
            CdpRunningValidator::new(get_default_config(), send);
        let rdh_mem_pos = 0x0; // RDH size is 64 bytes

        validator.set_current_rdh(&CORRECT_RDH_CRU_V7, rdh_mem_pos); // Data format is 2
        validator.check(&raw_data_tdt);
        validator.check(&raw_data_tdt_next);

        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    &*msg,
                    "0x40: [E30] ID is not 0xE0: 0xF1  [00 00 00 00 00 00 00 00 01 F1]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    &*msg,
                    "0x4A: [E40] ID is not 0xE8: 0xF2  [00 00 00 00 00 00 00 00 01 F2]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
        // No more errors
        assert!(stats_recv_ch.try_recv().is_err());
    }

    #[test]
    fn test_expect_ihw_invalidate_tdh_and_next_next() {
        const _VALID_ID: u8 = 0xF0;
        // Boring but very typical TDT, everything is 0 except for packet_done
        let raw_data_tdt = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF1];
        let raw_data_tdt_next = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF2];
        let raw_data_tdt_next_next = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF3];

        let (send, stats_recv_ch) = flume::unbounded();

        let mut validator: CdpRunningValidator<RdhCru, MockConfig> =
            CdpRunningValidator::new(get_running_checks_config(), send);
        let rdh_mem_pos = 0x0; // RDH size is 64 bytes

        validator.set_current_rdh(&CORRECT_RDH_CRU_V7, rdh_mem_pos); // Data format is 2
        validator.check(&raw_data_tdt);
        validator.check(&raw_data_tdt_next);
        validator.check(&raw_data_tdt_next_next);

        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    &*msg,
                    "0x40: [E30] ID is not 0xE0: 0xF1  [00 00 00 00 00 00 00 00 01 F1]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    &*msg,
                    "0x4A: [E40] ID is not 0xE8: 0xF2  [00 00 00 00 00 00 00 00 01 F2]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    &*msg,
                    "0x4A: [E444] TDH trigger_orbit is not equal to RDH orbit [00 00 00 00 00 00 00 00 01 F2]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                // Amibiguous error, could be several different data words
                assert_eq!(
                    &*msg,
                    "0x54: [E991] Unrecognized ID in ITS payload, could be Data Word/TDT/CDW based on current state, attempting to parse as Data Word [00 00 00 00 00 00 00 00 01 F3]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                // Amibiguous error, could be several different data words
                assert_eq!(
                    &*msg,
                    "0x54: [E70] ID is invalid: 0xF3 [00 00 00 00 00 00 00 00 01 F3]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
        // No more errors
        assert!(stats_recv_ch.try_recv().is_err());
    }
}
