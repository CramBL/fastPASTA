#![allow(non_camel_case_types)] // An exception to the Rust naming convention, for the state machine macro types

use crate::{
    util::stats::StatType,
    validators::status_words::STATUS_WORD_SANITY_CHECKER,
    words::{
        rdh::RdhCRUv7,
        rdh::RDH,
        status_words::{Ddw0, Ihw, StatusWord, Tdh, Tdt},
    },
    ByteSlice,
};

use log::debug;
use sm::sm;

use self::CDP_PAYLOAD_FSM_Continuous::{
    c_DATA_, c_IHW_, c_TDH_, DDW0_or_TDH_, DATA_, DDW0_, IHW_, TDH_,
};
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

const MAX_WORDS_PAYLOAD: u16 = 508; // 512 GBT word - 4 from RDH

pub struct CdpRunningValidator {
    sm: CDP_PAYLOAD_FSM_Continuous::Variant,
    pub current_rdh: Option<RdhCRUv7>,
    current_ihw: Option<Ihw>,
    current_tdh: Option<Tdh>,
    current_tdt: Option<Tdt>,
    current_ddw0: Option<Ddw0>,
    last_rdh: Option<RdhCRUv7>,
    last_ihw: Option<Ihw>,
    last_tdh: Option<Tdh>,
    last_tdt: Option<Tdt>,
    last_ddw0: Option<Ddw0>,
    error_count: u8,
    gbt_word_counter: u16,
    stats_send_ch: std::sync::mpsc::Sender<StatType>,
}
impl CdpRunningValidator {
    pub fn new(stats_send_ch: std::sync::mpsc::Sender<StatType>) -> Self {
        Self {
            sm: CDP_PAYLOAD_FSM_Continuous::Machine::new(IHW_).as_enum(),
            current_rdh: None,
            current_ihw: None,
            current_tdh: None,
            current_tdt: None,
            current_ddw0: None,
            last_rdh: None,
            last_ihw: None,
            last_tdh: None,
            last_tdt: None,
            last_ddw0: None,
            error_count: 0,
            gbt_word_counter: 0,
            stats_send_ch,
        }
    }

    pub fn reset_fsm(&mut self) {
        log::warn!("Resetting CDP Payload FSM");
        self.sm = CDP_PAYLOAD_FSM_Continuous::Machine::new(IHW_).as_enum();
    }

    /// Takes a slice of bytes wrapped in an enum of the expected status word then:
    /// 1. Deserializes the slice as the expected status word and checks it for sanity.
    /// 2. If the sanity check fails, the error is printed to stderr and returned as an error.
    /// 3. Stores the deserialized status word as the last status word of the same type.
    fn process_status_word(&mut self, status_word: StatusWordKind) -> Result<(), ()> {
        let mut result = Ok(());
        match status_word {
            StatusWordKind::Ihw(ihw) => {
                let ihw = Ihw::load(&mut ihw.clone()).unwrap();
                debug!("{ihw}");
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_ihw(&ihw) {
                    self.stats_send_ch
                        .send(StatType::Error(
                            String::from("IHW sanity check failed:") + &e,
                        ))
                        .unwrap();
                    debug!("IHW: {:X?}", ihw.to_byte_slice());
                    result = Err(());
                }
                self.gbt_word_counter = 1; // Reset the GBT word counter
                self.current_ihw = Some(ihw);
            }
            StatusWordKind::Tdh(tdh) => {
                let tdh = Tdh::load(&mut tdh.clone()).unwrap();
                debug!("{tdh}");
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_tdh(&tdh) {
                    self.stats_send_ch
                        .send(StatType::Error(format!("TDH sanity check failed: {}", e)))
                        .unwrap();
                    debug!("TDH: {:X?}", tdh.to_byte_slice());
                    result = Err(());
                }
                self.current_tdh = Some(tdh);
            }
            StatusWordKind::Tdt(tdt) => {
                let tdt = Tdt::load(&mut tdt.clone()).unwrap();
                debug!("{tdt}");
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_tdt(&tdt) {
                    self.stats_send_ch
                        .send(StatType::Error(format!("TDT sanity check failed: {}", e)))
                        .unwrap();
                    print!("{}", e);
                    debug!("TDT: {:X?}", tdt.to_byte_slice());
                    result = Err(());
                }
                self.current_tdt = Some(tdt);
            }
            StatusWordKind::Ddw0(ddw0) => {
                let ddw0 = Ddw0::load(&mut ddw0.clone()).unwrap();
                debug!("{ddw0}");
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_ddw0(&ddw0) {
                    self.stats_send_ch
                        .send(StatType::Error(format!("DDW0 sanity check failed: {}", e)))
                        .unwrap();
                    debug!("DDW0: {:X?}", ddw0.to_byte_slice());
                    result = Err(());
                }
                if self.current_rdh.as_ref().unwrap().rdh2.stop_bit != 1 {
                    self.stats_send_ch
                        .send(StatType::Error(
                            "DDW0 found but RDH stop bit is not set".to_string(),
                        ))
                        .unwrap();
                    debug!("DDW0: {:X?}", ddw0.to_byte_slice());
                }
                if self.current_rdh.as_ref().unwrap().rdh2.pages_counter == 0 {
                    self.stats_send_ch
                        .send(StatType::Error(
                            "DDW0 found but RDH page counter is 0".to_string(),
                        ))
                        .unwrap();
                    debug!("DDW0: {:X?}", ddw0.to_byte_slice());
                }
                self.current_ddw0 = Some(ddw0);
            }
        }
        result
    }

    pub fn check(&mut self, rdh: &RdhCRUv7, gbt_word: &[u8]) -> Result<(), String> {
        debug_assert!(gbt_word.len() == 10);
        self.gbt_word_counter += 1; // Tracks the number of GBT words seen in the current CDP

        use CDP_PAYLOAD_FSM_Continuous::Variant::*;
        use CDP_PAYLOAD_FSM_Continuous::*;

        if self.current_rdh.is_none() {
            let tmp_rdh = RdhCRUv7::load(&mut rdh.to_byte_slice()).unwrap();
            self.current_rdh = Some(tmp_rdh);
        }

        let current_st = self.sm.clone();

        let nxt_st = match current_st {
            InitialIHW_(m) => {
                debug_assert!(rdh.rdh2.stop_bit == 0);
                debug_assert!(rdh.rdh2.pages_counter == 0);
                if let Err(_) = self.process_status_word(StatusWordKind::Ihw(gbt_word)) {
                    self.error_count += 1;
                }
                m.transition(_WasIhw).as_enum()
            }

            TDH_By_WasIhw(m) => {
                if let Err(_) = self.process_status_word(StatusWordKind::Tdh(gbt_word)) {
                    self.error_count += 1;
                }
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
                    if let Err(_) = self.process_status_word(StatusWordKind::Tdh(gbt_word)) {
                        self.error_count += 1;
                    }
                    debug_assert!(self.current_tdh.as_ref().unwrap().no_data() == 0);
                    m.transition(_NoDataFalse).as_enum()
                } else {
                    debug_assert!(gbt_word[9] == 0xE4);
                    // DDW0
                    if let Err(_) = self.process_status_word(StatusWordKind::Ddw0(gbt_word)) {
                        self.error_count += 1;
                    }
                    m.transition(_WasDdw0).as_enum()
                }
            }

            DATA_By_NoDataFalse(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    if let Err(_) = self.process_status_word(StatusWordKind::Tdt(gbt_word)) {
                        self.error_count += 1;
                    }
                    // Next word is decided by if packet_done is 0 or 1
                    // `current_tdt` is used as processing the status words overrides the previous tdt
                    match self.current_tdt.as_ref().unwrap().packet_done() {
                        false => m.transition(_WasTDTpacketDoneFalse).as_enum(),
                        true => m.transition(_WasTDTpacketDoneTrue).as_enum(),
                    }
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(_WasData).as_enum()
                }
            }

            DATA_By_WasData(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    if let Err(_) = self.process_status_word(StatusWordKind::Tdt(gbt_word)) {
                        self.error_count += 1;
                    }
                    // Next word is decided by if packet_done is 0 or 1
                    // `current_tdt` is used as processing the status words overrides the previous tdt
                    match self.current_tdt.as_ref().unwrap().packet_done() {
                        false => m.transition(_WasTDTpacketDoneFalse).as_enum(),
                        true => m.transition(_WasTDTpacketDoneTrue).as_enum(),
                    }
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(_WasData).as_enum()
                }
            }

            c_IHW_By_WasTDTpacketDoneFalse(m) => {
                if let Err(_) = self.process_status_word(StatusWordKind::Ihw(gbt_word)) {
                    self.error_count += 1;
                }
                m.transition(_Next).as_enum()
            }

            c_TDH_By_Next(m) => {
                if let Err(_) = self.process_status_word(StatusWordKind::Tdh(gbt_word)) {
                    self.error_count += 1;
                }
                if self.current_tdh.as_ref().unwrap().continuation() != 1 {
                    self.error_count += 1;
                    self.stats_send_ch
                        .send(StatType::Error("Tdh continuation is not 1".to_string()))
                        .unwrap();
                }
                m.transition(_Next).as_enum()
            }

            c_DATA_By_Next(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    if let Err(_) = self.process_status_word(StatusWordKind::Tdt(gbt_word)) {
                        self.error_count += 1;
                    }
                    match self.current_tdt.as_ref().unwrap().packet_done() {
                        false => m.transition(_WasTDTpacketDoneFalse).as_enum(),
                        true => m.transition(_WasTDTpacketDoneTrue).as_enum(),
                    }
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(_WasData).as_enum()
                }
            }

            c_DATA_By_WasData(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    if let Err(_) = self.process_status_word(StatusWordKind::Tdt(gbt_word)) {
                        self.error_count += 1;
                    }
                    match self.current_tdt.as_ref().unwrap().packet_done() {
                        false => m.transition(_WasTDTpacketDoneFalse).as_enum(),
                        true => m.transition(_WasTDTpacketDoneTrue).as_enum(),
                    }
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(_WasData).as_enum()
                }
            }

            DDW0_or_TDH_or_IHW_By_WasTDTpacketDoneTrue(m) => {
                if gbt_word[9] == 0xE8 {
                    // TDH
                    if let Err(_) = self.process_status_word(StatusWordKind::Tdh(gbt_word)) {
                        self.error_count += 1;
                    }
                    if self.current_tdh.as_ref().unwrap().internal_trigger() != 1 {
                        self.error_count += 1;
                        self.stats_send_ch
                            .send(StatType::Error(format!(
                                "TDH internal trigger is not 1, got: {:02X?}",
                                gbt_word
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
                    if let Err(_) = self.process_status_word(StatusWordKind::Ddw0(gbt_word)) {
                        self.error_count += 1;
                    }
                    m.transition(_WasDdw0).as_enum()
                } else {
                    // IHW
                    if let Err(_) = self.process_status_word(StatusWordKind::Ihw(gbt_word)) {
                        self.error_count += 1;
                    }
                    m.transition(_WasIhw).as_enum()
                }
            }
            IHW_By_WasDdw0(m) => {
                debug_assert!(rdh.rdh2.stop_bit == 0);
                debug_assert!(rdh.rdh2.pages_counter == 0);
                if let Err(_) = self.process_status_word(StatusWordKind::Ihw(gbt_word)) {
                    self.error_count += 1;
                }
                m.transition(_WasIhw).as_enum()
            }
        };

        self.sm = nxt_st;
        Ok(())
    }
}
