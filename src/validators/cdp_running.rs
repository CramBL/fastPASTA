#![allow(non_camel_case_types)] // An exception to the Rust naming convention, for the state machine macro types

use crate::{
    util::stats_controller::StatType,
    validators::status_words::STATUS_WORD_SANITY_CHECKER,
    words::{
        rdh::RDH,
        status_words::{Ddw0, Ihw, StatusWord, Tdh, Tdt},
    },
    ByteSlice,
};

use log::debug;
use sm::sm;

use self::CDP_PAYLOAD_FSM_Continuous::IHW_;

use super::data_words::DATA_WORD_SANITY_CHECKER;
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
        }
    }

    pub fn reset_fsm(&mut self) {
        log::warn!("Resetting CDP Payload FSM");
        self.sm = CDP_PAYLOAD_FSM_Continuous::Machine::new(IHW_).as_enum();
    }

    pub fn set_current_rdh(&mut self, rdh: &T, payload_mem_pos: u64) {
        self.current_rdh = Some(T::load(&mut rdh.to_byte_slice()).unwrap());
        self.gbt_word_counter = 0;
        self.payload_mem_pos = payload_mem_pos;
    }

    pub fn check(&mut self, gbt_word: &[u8]) {
        debug_assert!(gbt_word.len() == 10);
        self.gbt_word_counter += 1; // Tracks the number of GBT words seen in the current CDP

        use CDP_PAYLOAD_FSM_Continuous::Variant::*;
        use CDP_PAYLOAD_FSM_Continuous::*;

        let current_st = self.sm.clone();

        let nxt_st = match current_st {
            InitialIHW_(m) => {
                debug_assert!(self.current_rdh.as_ref().unwrap().stop_bit() == 0);
                debug_assert!(self.current_rdh.as_ref().unwrap().pages_counter() == 0);
                self.process_status_word(StatusWordKind::Ihw(gbt_word));
                m.transition(_WasIhw).as_enum()
            }

            TDH_By_WasIhw(m) => {
                self.process_status_word(StatusWordKind::Tdh(gbt_word));
                debug_assert!(self.current_tdh.as_ref().unwrap().continuation() == 0);
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
                    // DDW0
                    self.process_status_word(StatusWordKind::Ddw0(gbt_word));

                    m.transition(_WasDdw0).as_enum()
                }
            }

            DATA_By_NoDataFalse(m) => {
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
                if self.current_tdh.as_ref().unwrap().continuation() != 1 {
                    self.stats_send_ch
                        .send(StatType::Error("Tdh continuation is not 1".to_string()))
                        .unwrap();
                }
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
                    if self.current_tdh.as_ref().unwrap().internal_trigger() != 1 {
                        self.stats_send_ch
                            .send(StatType::Error(format!(
                                "TDH internal trigger is not 1, got: {gbt_word:02X?}"
                            )))
                            .unwrap();
                        let tmp_rdh = self.current_rdh.as_ref().unwrap();
                        debug!("{tmp_rdh}");
                    }
                    debug_assert!(self.current_tdh.as_ref().unwrap().continuation() == 0);
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
                    m.transition(_WasIhw).as_enum()
                }
            }
            IHW_By_WasDdw0(m) => {
                debug_assert!(self.current_rdh.as_ref().unwrap().stop_bit() == 0);
                debug_assert!(self.current_rdh.as_ref().unwrap().pages_counter() == 0);
                self.process_status_word(StatusWordKind::Ihw(gbt_word));

                m.transition(_WasIhw).as_enum()
            }
        };

        self.sm = nxt_st;
    }

    /// Takes a slice of bytes wrapped in an enum of the expected status word then:
    /// 1. Deserializes the slice as the expected status word and checks it for sanity.
    /// 2. If the sanity check fails, the error is sent to the stats channel
    /// 3. Stores the deserialized status word as the last status word of the same type.
    #[inline(always)]
    fn process_status_word(&mut self, status_word: StatusWordKind) {
        match status_word {
            StatusWordKind::Ihw(ihw) => {
                let ihw = Ihw::load(&mut <&[u8]>::clone(&ihw)).unwrap();
                debug!("{ihw}");
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_ihw(&ihw) {
                    let mem_pos = (self.gbt_word_counter as u64 * 80) + self.payload_mem_pos;
                    self.stats_send_ch
                        .send(StatType::Error(format!("{mem_pos:#X}: [E00] {e}")))
                        .unwrap();
                    debug!("IHW: {ihw}");
                }
                self.current_ihw = Some(ihw);
            }
            StatusWordKind::Tdh(tdh) => {
                let tdh = Tdh::load(&mut <&[u8]>::clone(&tdh)).unwrap();
                debug!("{tdh}");
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_tdh(&tdh) {
                    let mem_pos = (self.gbt_word_counter as u64 * 80) + self.payload_mem_pos;
                    self.stats_send_ch
                        .send(StatType::Error(format!("{mem_pos:#X}: [E00] {e}")))
                        .unwrap();
                    debug!("TDH: {tdh}");
                }
                self.current_tdh = Some(tdh);
            }
            StatusWordKind::Tdt(tdt) => {
                let tdt = Tdt::load(&mut <&[u8]>::clone(&tdt)).unwrap();
                debug!("{tdt}");
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_tdt(&tdt) {
                    let mem_pos = (self.gbt_word_counter as u64 * 80) + self.payload_mem_pos;
                    self.stats_send_ch
                        .send(StatType::Error(format!("{mem_pos:#X}: [E00] {e}")))
                        .unwrap();
                    print!("{e}");
                    debug!("TDT: {tdt}");
                }
                self.current_tdt = Some(tdt);
            }
            StatusWordKind::Ddw0(ddw0) => {
                let ddw0 = Ddw0::load(&mut <&[u8]>::clone(&ddw0)).unwrap();
                debug!("{ddw0}");
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_ddw0(&ddw0) {
                    let mem_pos = (self.gbt_word_counter as u64 * 80) + self.payload_mem_pos;
                    self.stats_send_ch
                        .send(StatType::Error(format!("{mem_pos:#X}: [E00] {e}")))
                        .unwrap();
                    debug!("DDW0: {ddw0}");
                }
                if self.current_rdh.as_ref().unwrap().stop_bit() != 1 {
                    let mem_pos = (self.gbt_word_counter as u64 * 80) + self.payload_mem_pos;
                    self.stats_send_ch
                        .send(StatType::Error(format!(
                            "{mem_pos:#X}: [E10] DDW0 received but RDH stop bit is not 1"
                        )))
                        .unwrap();
                    debug!("DDW0: {:X?}", ddw0.to_byte_slice());
                }
                if self.current_rdh.as_ref().unwrap().pages_counter() == 0 {
                    let mem_pos = (self.gbt_word_counter as u64 * 80) + self.payload_mem_pos;
                    self.stats_send_ch
                        .send(StatType::Error(format!(
                            "{mem_pos:#X}: [E11] DDW0 found but RDH page counter is 0"
                        )))
                        .unwrap();
                    debug!("DDW0: {:X?}", ddw0.to_byte_slice());
                }
                self.current_ddw0 = Some(ddw0);
            }
        }
    }

    /// Takes a slice of bytes expected to be a data word, and checks if it has a valid identifier.
    #[inline(always)]
    fn process_data_word(&mut self, data_word: &[u8]) {
        if let Err(e) = DATA_WORD_SANITY_CHECKER.check_any(data_word) {
            let mem_pos = (self.gbt_word_counter as u64 * 80) + self.payload_mem_pos;
            self.stats_send_ch
                .send(StatType::Error(format!("{mem_pos:#X}: [E02] {e}")))
                .unwrap();
            debug!("Data word: {data_word:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::words::rdh::*;
    #[test]
    fn test_validate_ihw() {
        const VALID_ID: u8 = 0xE0;
        const _ACTIVE_LANES_14_ACTIVE: u32 = 0x3F_FF;
        let raw_data_ihw = [
            0xFF, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, VALID_ID,
        ];

        let (send, stats_recv_ch) = std::sync::mpsc::channel();
        let mut validator = CdpRunningValidator::<RdhCRUv7>::new(send);
        let payload_mem_pos = 512;

        validator.set_current_rdh(&CORRECT_RDH_CRU_V7, payload_mem_pos);
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
        let mut validator = CdpRunningValidator::<RdhCRUv7>::new(send);
        let payload_mem_pos = 512;

        validator.set_current_rdh(&CORRECT_RDH_CRU_V7, payload_mem_pos);
        validator.check(&raw_data_ihw);

        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    msg,
                    "0x250: [E00] ID is not 0xE0: 0xE1 Full Word: E1 00 00 00 00 00 00 00 3F FF [79:0]"
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
        let mut validator = CdpRunningValidator::<RdhCRUv7>::new(send);
        let payload_mem_pos = 512;

        validator.set_current_rdh(&CORRECT_RDH_CRU_V7, payload_mem_pos);
        validator.check(&raw_data_tdt);

        match stats_recv_ch.recv() {
            Ok(StatType::Error(msg)) => {
                assert_eq!(
                    msg,
                    "0x250: [E00] ID is not 0xE0: 0xF1 Full Word: F1 01 00 00 00 00 00 00 00 00 [79:0]"
                );
                println!("{msg}");
            }
            _ => unreachable!(),
        }
    }
}
