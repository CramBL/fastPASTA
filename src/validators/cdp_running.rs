use crate::{
    validators::status_words::STATUS_WORD_SANITY_CHECKER,
    words::{
        rdh::RdhCRUv7,
        status_words::{Ddw0, Ihw, StatusWord, Tdh, Tdt},
    },
    ByteSlice, GbtWord, RDH,
};

use sm::sm;

use self::CDP_PAYLOAD_FSM_Continuous::IhwSt;
sm! {
    // All states have the '-St' suffix
    // The statemachine macro notation goes like this:
    // EventName { StatesFrom => StateTo }
    CDP_PAYLOAD_FSM_Continuous {

        InitialStates { IhwSt }

        // No value is associated with the event
        Next {
            IhwSt => TdhSt,
            CihwSt => CtdhSt,
            CtdhSt => CdataSt, // TDH should have continuation bit set
            CtdtSt => Ddw0St, // TDT Should have packet_done == 1
            Ddw0St => IhwSt,
            DataSt => IhwSt // TDT with packet done and CDP is full (offset == 5088)
        }

        NoDataTrue {
            TdhSt => Ddw0OrTdhSt
        }

        NoDataFalse {
            TdhSt => DataSt
        }

        WasTdh {
            Ddw0OrTdhSt => DataSt
        }

        WasDdw0 {
            Ddw0OrTdhSt => Ddw0St
        }

        WasData {
            DataSt => DataSt,
            CdataSt => CdataSt
        }

        WasTDTpacketDoneFalse {
            // Event Page Should be full
            DataSt => CihwSt
        }

        WasTDTPacketDoneTrue {
            DataSt => TdhSt, // Next TDH should have internal trigger set
            CdataSt => Ddw0St
        }

        WasTDTandHBa {
            DataSt => Ddw0St,
            CdataSt => Ddw0St
        }
    }
}
enum StatusWordKind<'a> {
    Ihw(&'a [u8]),
    Tdh(&'a [u8]),
    Tdt(&'a [u8]),
    Ddw0(&'a [u8]),
}

pub struct CdpRunningValidator {
    sm: CDP_PAYLOAD_FSM_Continuous::Variant,
    last_rdh: Option<RdhCRUv7>,
    last_ihw: Option<Ihw>,
    last_tdh: Option<Tdh>,
    last_tdt: Option<Tdt>,
    last_ddw0: Option<Ddw0>,
    error_count: u8,
}
impl CdpRunningValidator {
    pub fn new() -> Self {
        Self {
            sm: CDP_PAYLOAD_FSM_Continuous::Machine::new(IhwSt).as_enum(),
            last_rdh: None,
            last_ihw: None,
            last_tdh: None,
            last_tdt: None,
            last_ddw0: None,
            error_count: 0,
        }
    }

    /// Takes a slice of bytes wrapped in an enum of the expected status word then:
    /// 1. Deserializes the slice as the expected status word and checks it for sanity.
    /// 2. If the sanity check fails, the error is printed to stderr and returned as an error.
    /// 3. Stores the deserialized status word as the last status word of the same type.
    fn process_status_word(&mut self, status_word: StatusWordKind) -> Result<(), String> {
        let mut result = Ok(());
        match status_word {
            StatusWordKind::Ihw(ihw) => {
                let ihw = Ihw::load(&mut ihw.clone()).unwrap();
                ihw.print();
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_ihw(&ihw) {
                    eprintln!("IHW sanity check failed: {}", &e);
                    eprintln!("IHW: {:X?}", ihw.to_byte_slice());
                    result = Err(e);
                }
                self.last_ihw = Some(ihw);
            }
            StatusWordKind::Tdh(tdh) => {
                let tdh = Tdh::load(&mut tdh.clone()).unwrap();
                tdh.print();
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_tdh(&tdh) {
                    eprintln!("TDH sanity check failed: {}", &e);
                    eprintln!("TDH: {:X?}", tdh.to_byte_slice());
                    result = Err(e);
                }
                self.last_tdh = Some(tdh);
            }
            StatusWordKind::Tdt(tdt) => {
                let tdt = Tdt::load(&mut tdt.clone()).unwrap();
                tdt.print();
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_tdt(&tdt) {
                    eprintln!("TDT sanity check failed: {}", &e);
                    eprintln!("TDT: {:X?}", tdt.to_byte_slice());
                    result = Err(e);
                }
                self.last_tdt = Some(tdt);
            }
            StatusWordKind::Ddw0(ddw0) => {
                let ddw0 = Ddw0::load(&mut ddw0.clone()).unwrap();
                ddw0.print();
                if let Err(e) = STATUS_WORD_SANITY_CHECKER.sanity_check_ddw0(&ddw0) {
                    eprintln!("DDW0 sanity check failed: {}", &e);
                    eprintln!("DDW0: {:X?}", ddw0.to_byte_slice());
                    result = Err(e);
                }
                self.last_ddw0 = Some(ddw0);
            }
        }
        result
    }

    pub fn check(&mut self, rdh: &RdhCRUv7, gbt_word: &[u8]) -> Result<(), String> {
        debug_assert!(gbt_word.len() == 10);
        use CDP_PAYLOAD_FSM_Continuous::Variant::*;
        use CDP_PAYLOAD_FSM_Continuous::*;

        if self.last_rdh.is_none() {
            let tmp_rdh = RdhCRUv7::load(&mut rdh.to_byte_slice()).unwrap();
            self.last_rdh = Some(tmp_rdh);
        }

        let current_st = self.sm.clone();

        let nxt_st = match current_st {
            InitialIhwSt(m) => {
                debug_assert!(rdh.rdh2.stop_bit == 0);
                debug_assert!(rdh.rdh2.pages_counter == 0);
                if let Err(e) = self.process_status_word(StatusWordKind::Ihw(gbt_word)) {
                    eprintln!("Error: {}", e);
                    self.error_count += 1;
                }
                m.transition(Next).as_enum()
            }

            TdhStByNext(m) => {
                if let Err(e) = self.process_status_word(StatusWordKind::Tdh(gbt_word)) {
                    eprintln!("Error: {}", e);
                    self.error_count += 1;
                }
                debug_assert!(self.last_tdh.as_ref().unwrap().continuation() == 0);
                match self.last_tdh.as_ref().unwrap().no_data() {
                    0 => m.transition(NoDataFalse).as_enum(),
                    1 => m.transition(NoDataTrue).as_enum(),
                    _ => unreachable!(),
                }
            }
            CtdhStByNext(m) => {
                if let Err(e) = self.process_status_word(StatusWordKind::Tdh(gbt_word)) {
                    eprintln!("Error: {}", e);
                    self.error_count += 1;
                }
                debug_assert!(self.last_tdh.as_ref().unwrap().continuation() == 1);
                m.transition(Next).as_enum()
            }
            CdataStByNext(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    if let Err(_) = self.process_status_word(StatusWordKind::Tdt(gbt_word)) {
                        self.error_count += 1;
                    }
                    debug_assert!(self.last_tdt.as_ref().unwrap().packet_done());
                    m.transition(WasTDTPacketDoneTrue).as_enum()
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(WasData).as_enum()
                }
            }
            Ddw0StByNext(m) => {
                debug_assert!(rdh.rdh2.stop_bit == 1);
                debug_assert!(rdh.rdh2.pages_counter >= 1);
                if let Err(_) = self.process_status_word(StatusWordKind::Ddw0(gbt_word)) {
                    self.error_count += 1;
                }
                m.transition(Next).as_enum()
            }
            IhwStByNext(m) => {
                if let Err(_) = self.process_status_word(StatusWordKind::Ihw(gbt_word)) {
                    self.error_count += 1;
                }
                m.transition(Next).as_enum()
            }
            Ddw0OrTdhStByNoDataTrue(m) => {
                if gbt_word[9] == 0xE8 {
                    // TDH
                    if let Err(e) = self.process_status_word(StatusWordKind::Tdh(gbt_word)) {
                        eprintln!("Error: {}", e);
                        self.error_count += 1;
                    }
                    debug_assert!(self.last_tdh.as_ref().unwrap().no_data() == 0);
                    m.transition(WasTdh).as_enum()
                } else {
                    debug_assert!(gbt_word[9] == 0xE4);
                    // DDW0
                    if let Err(_) = self.process_status_word(StatusWordKind::Ddw0(gbt_word)) {
                        self.error_count += 1;
                    }
                    m.transition(WasDdw0).as_enum()
                }
            }
            DataStByNoDataFalse(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    if let Err(_) = self.process_status_word(StatusWordKind::Tdt(gbt_word)) {
                        self.error_count += 1;
                    }
                    m.transition(WasTDTPacketDoneTrue).as_enum()
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(WasData).as_enum()
                }
            }
            DataStByWasData(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    if let Err(_) = self.process_status_word(StatusWordKind::Tdt(gbt_word)) {
                        self.error_count += 1;
                    }
                    debug_assert!(self.last_tdt.as_ref().unwrap().packet_done());
                    if rdh.rdh2.trigger_type & 0b10 == 0b10 && rdh.rdh2.pages_counter == 0 {
                        // TDT and HBA
                        if rdh.offset_new_packet == 5088 {
                            m.transition(Next).as_enum()
                        } else {
                            m.transition(WasTDTandHBa).as_enum()
                        }
                    } else {
                        if self.last_tdh.as_ref().unwrap().trigger_type() == 0 {
                            // from tdt to ddw0
                            m.transition(WasTDTandHBa).as_enum()
                        } else {
                            m.transition(WasTDTPacketDoneTrue).as_enum()
                        }
                    }
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(WasData).as_enum()
                }
            }
            CdataStByWasData(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    if let Err(_) = self.process_status_word(StatusWordKind::Tdt(gbt_word)) {
                        self.error_count += 1;
                    }
                    debug_assert!(self.last_tdt.as_ref().unwrap().packet_done());
                    m.transition(WasTDTPacketDoneTrue).as_enum()
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(WasData).as_enum()
                }
            }
            CihwStByWasTDTpacketDoneFalse(m) => {
                if let Err(_) = self.process_status_word(StatusWordKind::Ihw(gbt_word)) {
                    self.error_count += 1;
                }
                m.transition(Next).as_enum()
            }
            TdhStByWasTDTPacketDoneTrue(m) => {
                let tdh = Tdh::load(&mut gbt_word.clone()).unwrap();
                tdh.print();
                if tdh.internal_trigger() != 1 {
                    eprintln!("TDH internal trigger is not 1: {:02X?}", gbt_word);
                    self.last_tdh.as_ref().unwrap().print();
                    RdhCRUv7::print_header_text();
                    self.last_rdh.as_ref().unwrap().print();
                }
                debug_assert!(tdh.internal_trigger() == 1);
                let last_tdh_no_data = self.last_tdh.as_ref().unwrap().no_data();
                if let Err(e) = self.process_status_word(StatusWordKind::Tdh(gbt_word)) {
                    eprintln!("Error: {}", e);
                    self.error_count += 1;
                }
                match last_tdh_no_data {
                    0 => m.transition(NoDataFalse).as_enum(),
                    1 => m.transition(NoDataTrue).as_enum(),
                    _ => unreachable!(),
                }
            }
            Ddw0StByWasTDTPacketDoneTrue(m) => {
                if let Err(_) = self.process_status_word(StatusWordKind::Ddw0(gbt_word)) {
                    self.error_count += 1;
                }
                debug_assert!(rdh.rdh2.stop_bit == 1);
                debug_assert!(rdh.rdh2.pages_counter > 0);
                m.transition(Next).as_enum()
            }
            Ddw0StByWasTDTandHBa(m) => {
                if let Err(_) = self.process_status_word(StatusWordKind::Ddw0(gbt_word)) {
                    self.error_count += 1;
                }
                println!("%%%%%%%%%%%)");
                if rdh.rdh2.stop_bit == 0 {
                    rdh.rdh2.print();
                    eprint!("RDH2: {:X?} ", rdh.rdh2);
                }
                debug_assert!(rdh.rdh2.stop_bit == 1);
                debug_assert!(rdh.rdh2.pages_counter > 0);
                let trigger_type = rdh.rdh2.trigger_type;
                debug_assert!((trigger_type & 0b10) == 0b10);
                m.transition(Next).as_enum()
            }
            DataStByWasTdh(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    if let Err(_) = self.process_status_word(StatusWordKind::Tdt(gbt_word)) {
                        self.error_count += 1;
                    }
                    m.transition(WasTDTPacketDoneTrue).as_enum()
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(WasData).as_enum()
                }
            }
            Ddw0StByWasDdw0(m) => {
                if let Err(_) = self.process_status_word(StatusWordKind::Ddw0(gbt_word)) {
                    self.error_count += 1;
                }
                debug_assert!(rdh.rdh2.stop_bit == 1);
                debug_assert!(rdh.rdh2.pages_counter >= 1);
                m.transition(Next).as_enum()
            }
        };

        self.sm = nxt_st;

        Ok(())
    }
}
