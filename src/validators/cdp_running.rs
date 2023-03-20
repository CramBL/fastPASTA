#![allow(non_camel_case_types)] // An exception to the Rust naming convention, for the state machine macro types

use self::CDP_PAYLOAD_FSM_Continuous::IHW_;
use super::data_words::DATA_WORD_SANITY_CHECKER;
use crate::words::lib::RDH;
use crate::{
    stats::stats_controller::StatType,
    validators::status_words::STATUS_WORD_SANITY_CHECKER,
    words::status_words::{Ddw0, Ihw, StatusWord, Tdh, Tdt},
};
use sm::sm;
sm! {
    // All states have the '_' suffix and events have '_' prefix so they show up as `STATE_BY_EVENT` in the generated code
    // The statemachine macro notation goes like this:
    // EventName { StateFrom => StateTo }
    CDP_PAYLOAD_FSM_Continuous {

        InitialStates { IHW_ } // RDH: stop_bit == 0 && page == 0

        // No value is associated with the event
        _Next {
            c_IHW_ => c_TDH_, // RDH: stop_bit == 0 && page > 0
            c_TDH_ => c_DATA_ // TDH continuation bit set
        }

        _NoDataTrue {
            TDH_ => DDW0_or_TDH_,
            DDW0_or_TDH_ => DDW0_or_TDH_,
            DDW0_or_TDH_or_IHW_ => DDW0_or_TDH_
        }

        _NoDataFalse {
            TDH_ => DATA_,
            DDW0_or_TDH_ => DATA_,
            DDW0_or_TDH_or_IHW_ => DATA_
        }

        // end of CDP
        _WasDdw0 {
            DDW0_ => IHW_,
            DDW0_or_TDH_ => IHW_,
            DDW0_or_TDH_or_IHW_ => IHW_

        }

        _WasData {
            DATA_ => DATA_,
            c_DATA_ => c_DATA_
        }

        _WasTDTpacketDoneFalse {
            // Event Page Should be full (not strictly full to 512 GBT words apparently...)
            DATA_ => c_IHW_,
            c_DATA_ => c_IHW_
        }

        _WasTDTpacketDoneTrue {
            DATA_ => DDW0_or_TDH_or_IHW_, // If TDH: should have internal trigger set
                                          // If DDW0: RDH: stop_bit == 1 and Page > 0_
                                          // If IHW: Page > 0 && stop_bit == 0
            c_DATA_ => DDW0_or_TDH_or_IHW_ // RDH: stop_bit == 1 and Page > 0_
        }

        _WasIhw {
            IHW_ => TDH_,
            DDW0_or_TDH_or_IHW_ => TDH_
        }
    }
}
enum StatusWordKind<'a> {
    Ihw(&'a [u8]),
    Tdh(&'a [u8]),
    Tdt(&'a [u8]),
    Ddw0(&'a [u8]),
}

pub struct CdpRunningValidator<T: RDH> {
    sm: CDP_PAYLOAD_FSM_Continuous::Variant,
    current_rdh: Option<T>,
    current_ihw: Option<Ihw>,
    current_tdh: Option<Tdh>,
    current_tdt: Option<Tdt>,
    current_ddw0: Option<Ddw0>,
    gbt_word_counter: u16,
    stats_send_ch: std::sync::mpsc::Sender<StatType>,
    payload_mem_pos: u64,
    gbt_word_padding_size_bytes: u8,
}
impl<T: RDH> CdpRunningValidator<T> {
    pub fn new(stats_send_ch: std::sync::mpsc::Sender<StatType>) -> Self {
        Self {
            sm: CDP_PAYLOAD_FSM_Continuous::Machine::new(IHW_).as_enum(),
            current_rdh: None,
            current_ihw: None,
            current_tdh: None,
            current_tdt: None,
            current_ddw0: None,
            gbt_word_counter: 0,
            stats_send_ch,
            payload_mem_pos: 0,
            gbt_word_padding_size_bytes: 0,
        }
    }

    /// Helper function to format and report an error
    ///
    /// Takes in the error string slice and the word slice
    /// Adds the current memory position to the error string
    /// Sends the error to the stats channel
    #[inline]
    fn report_error(&mut self, error: &str, word_slice: &[u8]) {
        let mem_pos = self.calc_current_word_mem_pos();
        self.stats_send_ch
            .send(StatType::Error(format!(
                "{mem_pos:#X}: {error} [{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}]",
                word_slice[0],
                word_slice[1],
                word_slice[2],
                word_slice[3],
                word_slice[4],
                word_slice[5],
                word_slice[6],
                word_slice[7],
                word_slice[8],
                word_slice[9],
                            )))
            .expect("Failed to send error to stats channel");
    }

    /// Resets the state machine to the initial state and logs a warning
    ///
    /// Use this if a payload format is invalid and the next payload can be processed from the initial state
    #[inline]
    pub fn reset_fsm(&mut self) {
        log::warn!("Resetting CDP Payload FSM");
        self.sm = CDP_PAYLOAD_FSM_Continuous::Machine::new(IHW_).as_enum();
    }

    /// This function has to be called for every RDH
    ///
    /// It defines what is valid, and is necessary to keep track of the memory position of each word
    /// It uses the RDH to determine size of padding
    #[inline]
    pub fn set_current_rdh(&mut self, rdh: &T, rdh_mem_pos: u64) {
        self.current_rdh = Some(T::load(&mut rdh.to_byte_slice()).unwrap());
        self.gbt_word_counter = 0;
        self.payload_mem_pos = rdh_mem_pos + 64;
        if rdh.data_format() == 0 {
            self.gbt_word_padding_size_bytes = 6;
        } else {
            self.gbt_word_padding_size_bytes = 0; // Data format 2
        }
    }

    pub fn check(&mut self, gbt_word: &[u8]) {
        debug_assert!(gbt_word.len() == 10);
        self.gbt_word_counter += 1; // Tracks the number of GBT words seen in the current CDP
        log::trace!(
            "Processing GBT word: {gbt_word:X?} in memory position {:#X}",
            self.calc_current_word_mem_pos(),
        );

        use CDP_PAYLOAD_FSM_Continuous::Variant::*;
        use CDP_PAYLOAD_FSM_Continuous::*;

        let current_st = self.sm.clone();

        let nxt_st = match current_st {
            InitialIHW_(m) => {
                self.process_status_word(StatusWordKind::Ihw(gbt_word));
                self.check_rdh_at_initial_ihw(gbt_word);
                m.transition(_WasIhw).as_enum()
            }

            TDH_By_WasIhw(m) => {
                self.process_status_word(StatusWordKind::Tdh(gbt_word));
                self.check_tdh_no_continuation(gbt_word);
                match self.current_tdh.as_ref().unwrap().no_data() {
                    0 => m.transition(_NoDataFalse).as_enum(),
                    1 => m.transition(_NoDataTrue).as_enum(),
                    _ => unreachable!(),
                }
            }

            DDW0_or_TDH_By_NoDataTrue(m) => {
                if gbt_word[9] == 0xE8 {
                    // TDH
                    self.process_status_word(StatusWordKind::Tdh(gbt_word));
                    debug_assert!(self.current_tdh.as_ref().unwrap().no_data() == 0);
                    m.transition(_NoDataFalse).as_enum()
                } else {
                    debug_assert!(gbt_word[9] == 0xE4);
                    self.process_status_word(StatusWordKind::Ddw0(gbt_word));

                    m.transition(_WasDdw0).as_enum()
                }
            }

            DATA_By_NoDataFalse(m) => {
                if gbt_word[9] == 0xF0 {
                    self.process_status_word(StatusWordKind::Tdt(gbt_word));

                    // Next word is decided by if packet_done is 0 or 1
                    // `current_tdt` is used as processing the status words overrides the previous tdt
                    match self.current_tdt.as_ref().unwrap().packet_done() {
                        false => m.transition(_WasTDTpacketDoneFalse).as_enum(),
                        true => m.transition(_WasTDTpacketDoneTrue).as_enum(),
                    }
                } else {
                    self.process_data_word(gbt_word);
                    m.transition(_WasData).as_enum()
                }
            }

            DATA_By_WasData(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    self.process_status_word(StatusWordKind::Tdt(gbt_word));
                    // Next word is decided by if packet_done is 0 or 1
                    // `current_tdt` is used as processing the status words overrides the previous tdt
                    match self.current_tdt.as_ref().unwrap().packet_done() {
                        false => m.transition(_WasTDTpacketDoneFalse).as_enum(),
                        true => m.transition(_WasTDTpacketDoneTrue).as_enum(),
                    }
                } else {
                    self.process_data_word(gbt_word);
                    m.transition(_WasData).as_enum()
                }
            }

            c_IHW_By_WasTDTpacketDoneFalse(m) => {
                self.process_status_word(StatusWordKind::Ihw(gbt_word));
                m.transition(_Next).as_enum()
            }

            c_TDH_By_Next(m) => {
                self.process_status_word(StatusWordKind::Tdh(gbt_word));
                self.check_tdh_continuation(gbt_word);

                m.transition(_Next).as_enum()
            }

            c_DATA_By_Next(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    self.process_status_word(StatusWordKind::Tdt(gbt_word));
                    match self.current_tdt.as_ref().unwrap().packet_done() {
                        false => m.transition(_WasTDTpacketDoneFalse).as_enum(),
                        true => m.transition(_WasTDTpacketDoneTrue).as_enum(),
                    }
                } else {
                    self.process_data_word(gbt_word);
                    m.transition(_WasData).as_enum()
                }
            }

            c_DATA_By_WasData(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    self.process_status_word(StatusWordKind::Tdt(gbt_word));
                    match self.current_tdt.as_ref().unwrap().packet_done() {
                        false => m.transition(_WasTDTpacketDoneFalse).as_enum(),
                        true => m.transition(_WasTDTpacketDoneTrue).as_enum(),
                    }
                } else {
                    self.process_data_word(gbt_word);
                    m.transition(_WasData).as_enum()
                }
            }

            DDW0_or_TDH_or_IHW_By_WasTDTpacketDoneTrue(m) => {
                if gbt_word[9] == 0xE8 {
                    // TDH
                    self.process_status_word(StatusWordKind::Tdh(gbt_word));
                    self.check_tdh_by_was_tdt_packet_done_true(gbt_word);

                    match self.current_tdh.as_ref().unwrap().no_data() {
                        0 => m.transition(_NoDataFalse).as_enum(),
                        1 => m.transition(_NoDataTrue).as_enum(),
                        _ => unreachable!(),
                    }
                } else if gbt_word[9] == 0xE4 {
                    // DDW0
                    self.process_status_word(StatusWordKind::Ddw0(gbt_word));
                    m.transition(_WasDdw0).as_enum()
                } else {
                    // IHW
                    self.process_status_word(StatusWordKind::Ihw(gbt_word));
                    self.check_rdh_at_initial_ihw(gbt_word);

                    m.transition(_WasIhw).as_enum()
                }
            }
            IHW_By_WasDdw0(m) => {
                self.process_status_word(StatusWordKind::Ihw(gbt_word));
                self.check_rdh_at_initial_ihw(gbt_word);
                m.transition(_WasIhw).as_enum()
            }
        };

        self.sm = nxt_st;
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
    #[inline]
    fn process_status_word(&mut self, status_word: StatusWordKind) {
        match status_word {
            StatusWordKind::Ihw(ihw_as_slice) => {
                let ihw = Ihw::load(&mut <&[u8]>::clone(&ihw_as_slice)).unwrap();
                log::debug!("{ihw}");
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_ihw(&ihw) {
                    self.report_error(&format!("[E30] {e}"), ihw_as_slice);
                }
                self.current_ihw = Some(ihw);
            }
            StatusWordKind::Tdh(tdh_as_slice) => {
                let tdh = Tdh::load(&mut <&[u8]>::clone(&tdh_as_slice)).unwrap();
                log::debug!("{tdh}");
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_tdh(&tdh) {
                    self.report_error(&format!("[E40] {e}"), tdh_as_slice);
                }
                self.current_tdh = Some(tdh);
            }
            StatusWordKind::Tdt(tdt_as_slice) => {
                let tdt = Tdt::load(&mut <&[u8]>::clone(&tdt_as_slice)).unwrap();
                log::debug!("{tdt}");
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_tdt(&tdt) {
                    self.report_error(&format!("[E50] {e}"), tdt_as_slice);
                }
                self.current_tdt = Some(tdt);
            }
            StatusWordKind::Ddw0(ddw0_as_slice) => {
                let ddw0 = Ddw0::load(&mut <&[u8]>::clone(&ddw0_as_slice)).unwrap();
                log::debug!("{ddw0}");
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_ddw0(&ddw0) {
                    self.report_error(&format!("[E60] {e}"), ddw0_as_slice);
                }

                // Additional state dependent checks on RDH
                self.check_rdh_at_ddw0(ddw0_as_slice);
                self.current_ddw0 = Some(ddw0);
            }
        }
    }

    /// Takes a slice of bytes expected to be a data word, and checks if it has a valid identifier.
    #[inline]
    fn process_data_word(&mut self, data_word: &[u8]) {
        if let Err(e) = DATA_WORD_SANITY_CHECKER.check_any(data_word) {
            self.report_error(&format!("[E70] {e}"), data_word);
            log::debug!("Data word: {data_word:?}");
        }
    }

    // Minor checks done in certain states

    /// Checks TDH trigger and continuation following a TDT packet_done = 1
    #[inline]
    fn check_tdh_by_was_tdt_packet_done_true(&mut self, tdh_slice: &[u8]) {
        if self.current_tdh.as_ref().unwrap().internal_trigger() != 1 {
            self.report_error("TDH internal trigger is not 1", tdh_slice);
            let tmp_rdh = self.current_rdh.as_ref().unwrap();
            log::debug!("{tmp_rdh}");
        }
        if self.current_tdh.as_ref().unwrap().continuation() != 0 {
            self.report_error(
                "TDH continuation is not 0 but previous TDT had packet_done = 1",
                tdh_slice,
            );
            let tmp_rdh = self.current_rdh.as_ref().unwrap();
            log::debug!("{tmp_rdh}");
        }
    }

    /// Checks RDH stop_bit and pages_counter when a DDW0 is observed
    #[inline]
    fn check_rdh_at_ddw0(&mut self, ddw0_slice: &[u8]) {
        if self.current_rdh.as_ref().unwrap().stop_bit() != 1 {
            self.report_error("[E11] DDW0 observed but RDH stop bit is not 1", ddw0_slice);
        }
        if self.current_rdh.as_ref().unwrap().pages_counter() == 0 {
            self.report_error("[E11] DDW0 observed but RDH page counter is 0", ddw0_slice);
        }
    }
    /// Checks RDH stop_bit and pages_counter when an initial IHW is observed (not IHW during continuation)
    #[inline]
    fn check_rdh_at_initial_ihw(&mut self, ihw_slice: &[u8]) {
        if self.current_rdh.as_ref().unwrap().stop_bit() != 0 {
            self.report_error("[E12] IHW observed but RDH stop bit is not 0", ihw_slice);
        }
        if self.current_rdh.as_ref().unwrap().pages_counter() != 0 {
            self.report_error(
                "[E12] IHW observed but RDH page counter is not 0",
                ihw_slice,
            );
        }
    }

    /// Checks TDH when continuation is expected (last TDT packet_done = 0)
    #[inline]
    fn check_tdh_continuation(&mut self, gbt_word: &[u8]) {
        if self.current_tdh.as_ref().unwrap().continuation() != 1 {
            self.report_error("[E41] TDH continuation is not 1", gbt_word);
        }
    }
    #[inline]
    fn check_tdh_no_continuation(&mut self, gbt_word: &[u8]) {
        if self.current_tdh.as_ref().unwrap().continuation() != 0 {
            self.report_error("[E42] TDH continuation is not 0", gbt_word);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::words::rdh_cru::{test_data::CORRECT_RDH_CRU_V7, RdhCRU, V7};
    #[test]
    fn test_validate_ihw() {
        const VALID_ID: u8 = 0xE0;
        const _ACTIVE_LANES_14_ACTIVE: u32 = 0x3F_FF;
        let raw_data_ihw = [
            0xFF, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, VALID_ID,
        ];

        let (send, stats_recv_ch) = std::sync::mpsc::channel();
        let mut validator = CdpRunningValidator::<RdhCRU<V7>>::new(send);
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

        let (send, stats_recv_ch) = std::sync::mpsc::channel();
        let mut validator = CdpRunningValidator::<RdhCRU<V7>>::new(send);
        let rdh_mem_pos = 0x0;

        validator.set_current_rdh(&CORRECT_RDH_CRU_V7, rdh_mem_pos);
        validator.check(&raw_data_ihw);

        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    msg,
                    "0x40: [E00] ID is not 0xE0: 0xE1 Full Word: FF 3F 00 00 00 00 00 00 00 E1 [79:0]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_expect_ihw_invalidate_tdh() {
        const _VALID_ID: u8 = 0xF0;
        // Boring but very typical TDT, everything is 0 except for packet_done
        let raw_data_tdt = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF1];

        let (send, stats_recv_ch) = std::sync::mpsc::channel();
        let mut validator = CdpRunningValidator::<RdhCRU<V7>>::new(send);
        let rdh_mem_pos = 0x0; // RDH size is 64 bytes

        validator.set_current_rdh(&CORRECT_RDH_CRU_V7, rdh_mem_pos); // Data format is 2
        validator.check(&raw_data_tdt);

        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    msg,
                    "0x40: [E00] ID is not 0xE0: 0xF1 Full Word: 00 00 00 00 00 00 00 00 01 F1 [79:0]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_expect_ihw_invalidate_tdh_and_next() {
        const _VALID_ID: u8 = 0xF0;
        // Boring but very typical TDT, everything is 0 except for packet_done
        let raw_data_tdt = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF1];
        let raw_data_tdt_next = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF2];

        let (send, stats_recv_ch) = std::sync::mpsc::channel();
        let mut validator = CdpRunningValidator::<RdhCRU<V7>>::new(send);
        let rdh_mem_pos = 0x0; // RDH size is 64 bytes

        validator.set_current_rdh(&CORRECT_RDH_CRU_V7, rdh_mem_pos); // Data format is 2
        validator.check(&raw_data_tdt);
        validator.check(&raw_data_tdt_next);

        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    msg,
                    "0x40: [E00] ID is not 0xE0: 0xF1 Full Word: 00 00 00 00 00 00 00 00 01 F1 [79:0]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    msg,
                    "0x4A: [E00] ID is not 0xE8: 0xF2 Full Word: 00 00 00 00 00 00 00 00 01 F2 [79:0]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_expect_ihw_invalidate_tdh_and_next_next() {
        const _VALID_ID: u8 = 0xF0;
        // Boring but very typical TDT, everything is 0 except for packet_done
        let raw_data_tdt = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF1];
        let raw_data_tdt_next = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF2];
        let raw_data_tdt_next_next = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF3];

        let (send, stats_recv_ch) = std::sync::mpsc::channel();
        let mut validator = CdpRunningValidator::<RdhCRU<V7>>::new(send);
        let rdh_mem_pos = 0x0; // RDH size is 64 bytes

        validator.set_current_rdh(&CORRECT_RDH_CRU_V7, rdh_mem_pos); // Data format is 2
        validator.check(&raw_data_tdt);
        validator.check(&raw_data_tdt_next);
        validator.check(&raw_data_tdt_next_next);

        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    msg,
                    "0x40: [E00] ID is not 0xE0: 0xF1 Full Word: 00 00 00 00 00 00 00 00 01 F1 [79:0]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    msg,
                    "0x4A: [E00] ID is not 0xE8: 0xF2 Full Word: 00 00 00 00 00 00 00 00 01 F2 [79:0]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                // Data word error
                assert_eq!(
                    msg,
                    "0x54: [E02] ID is invalid: 0xF3 Full Word: 00 00 00 00 00 00 00 00 01 F3 [79:0]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
    }
}
