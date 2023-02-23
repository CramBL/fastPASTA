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
            Ddw0St => IhwSt
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

pub struct CdpRunningValidator {
    sm: CDP_PAYLOAD_FSM_Continuous::Variant,
    last_rdh: Option<RdhCRUv7>,
    last_ihw: Option<Ihw>,
    last_tdh: Option<Tdh>,
    last_tdt: Option<Tdt>,
    last_ddw0: Option<Ddw0>,
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
        }
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
                let ihw = Ihw::load(&mut gbt_word.clone()).unwrap();
                ihw.print();
                let check = STATUS_WORD_SANITY_CHECKER.sanity_check_ihw(&ihw);
                if check.is_err() {
                    eprintln!("IHW sanity check failed: {}", check.err().unwrap());
                    eprintln!("IHW: {:02X?}", gbt_word);
                }
                self.last_ihw = Some(ihw);
                m.transition(Next).as_enum()
            }

            TdhStByNext(m) => {
                let tdh = Tdh::load(&mut gbt_word.clone()).unwrap();
                tdh.print();
                debug_assert!(tdh.continuation() == 0);
                let check = STATUS_WORD_SANITY_CHECKER.sanity_check_tdh(&tdh);
                if check.is_err() {
                    eprintln!("TDH sanity check failed: {}", check.err().unwrap());
                    eprintln!("TDH: {:02X?}", gbt_word);
                    eprintln!("State is now: {:?}", m);
                }
                self.last_tdh = Some(tdh);
                match self.last_tdh.as_ref().unwrap().no_data() {
                    0 => m.transition(NoDataFalse).as_enum(),
                    1 => m.transition(NoDataTrue).as_enum(),
                    _ => unreachable!(),
                }
            }
            CtdhStByNext(m) => {
                let tdh = Tdh::load(&mut gbt_word.clone()).unwrap();
                tdh.print();
                debug_assert!(tdh.continuation() == 1);
                let check = STATUS_WORD_SANITY_CHECKER.sanity_check_tdh(&tdh);
                if check.is_err() {
                    eprintln!("TDH sanity check failed: {}", check.err().unwrap());
                    eprintln!("TDH: {:02X?}", gbt_word);
                    eprintln!("State is now: {:?}", m);
                }
                self.last_tdh = Some(tdh);
                m.transition(Next).as_enum()
            }
            CdataStByNext(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    let tdt = Tdt::load(&mut gbt_word.clone()).unwrap();
                    tdt.print();
                    debug_assert!(tdt.packet_done());
                    let check = STATUS_WORD_SANITY_CHECKER.sanity_check_tdt(&tdt);
                    if check.is_err() {
                        eprintln!("TDT sanity check failed: {}", check.err().unwrap());
                        eprintln!("TDT: {:02X?}", gbt_word);
                    }
                    self.last_tdt = Some(tdt);
                    m.transition(WasTDTPacketDoneTrue).as_enum()
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(WasData).as_enum()
                }
            }
            Ddw0StByNext(m) => {
                debug_assert!(rdh.rdh2.stop_bit == 1);
                debug_assert!(rdh.rdh2.pages_counter >= 1);
                let ddw0 = Ddw0::load(&mut gbt_word.clone()).unwrap();
                ddw0.print();
                let check = STATUS_WORD_SANITY_CHECKER.sanity_check_ddw0(&ddw0);
                if check.is_err() {
                    eprintln!("DDW0 sanity check failed: {}", check.err().unwrap());
                    eprintln!("DDW0: {:02X?}", gbt_word);
                }
                self.last_ddw0 = Some(ddw0);
                m.transition(Next).as_enum()
            }
            IhwStByNext(m) => {
                let ihw = Ihw::load(&mut gbt_word.clone()).unwrap();
                ihw.print();
                let check = STATUS_WORD_SANITY_CHECKER.sanity_check_ihw(&ihw);
                if check.is_err() {
                    eprintln!("IHW sanity check failed: {}", check.err().unwrap());
                    eprintln!("IHW: {:02X?}", gbt_word);
                }
                m.transition(Next).as_enum()
            }
            Ddw0OrTdhStByNoDataTrue(m) => {
                if gbt_word[9] == 0xE8 {
                    // TDH
                    let tdh = Tdh::load(&mut gbt_word.clone()).unwrap();
                    tdh.print();
                    debug_assert!(tdh.no_data() == 0);
                    let check = STATUS_WORD_SANITY_CHECKER.sanity_check_tdh(&tdh);
                    if check.is_err() {
                        eprintln!("TDH sanity check failed: {}", check.err().unwrap());
                        eprintln!("TDH: {:02X?}", gbt_word);
                        tdh.print();
                        eprintln!("State is now: {:?}", m);
                    }
                    self.last_tdh = Some(tdh);
                    m.transition(WasTdh).as_enum()
                } else {
                    debug_assert!(gbt_word[9] == 0xE4);
                    // DDW0
                    let ddw0 = Ddw0::load(&mut gbt_word.clone()).unwrap();
                    ddw0.print();
                    let check = STATUS_WORD_SANITY_CHECKER.sanity_check_ddw0(&ddw0);
                    if check.is_err() {
                        eprintln!("DDW0 sanity check failed: {}", check.err().unwrap());
                        eprintln!("DDW0: {:02X?}", gbt_word);
                    }
                    self.last_ddw0 = Some(ddw0);
                    m.transition(WasDdw0).as_enum()
                }
            }
            DataStByNoDataFalse(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    let tdt = Tdt::load(&mut gbt_word.clone()).unwrap();
                    tdt.print();
                    let check = STATUS_WORD_SANITY_CHECKER.sanity_check_tdt(&tdt);
                    if check.is_err() {
                        eprintln!("TDT sanity check failed: {}", check.err().unwrap());
                        eprintln!("TDT: {:02X?}", gbt_word);
                    }
                    self.last_tdt = Some(tdt);
                    m.transition(WasTDTPacketDoneTrue).as_enum()
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(WasData).as_enum()
                }
            }
            DataStByWasData(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    let tdt = Tdt::load(&mut gbt_word.clone()).unwrap();
                    tdt.print();
                    let check = STATUS_WORD_SANITY_CHECKER.sanity_check_tdt(&tdt);
                    if check.is_err() {
                        eprintln!("TDT sanity check failed: {}", check.err().unwrap());
                        eprintln!("TDT: {:02X?}", gbt_word);
                    }
                    self.last_tdt = Some(tdt);
                    debug_assert!(self.last_tdt.as_ref().unwrap().packet_done());
                    if rdh.rdh2.trigger_type & 0b10 == 0b10 {
                        // TDT and HBA
                        m.transition(WasTDTandHBa).as_enum()
                    } else {
                        m.transition(WasTDTPacketDoneTrue).as_enum()
                    }
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(WasData).as_enum()
                }
            }
            CdataStByWasData(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    let tdt = Tdt::load(&mut gbt_word.clone()).unwrap();
                    tdt.print();
                    debug_assert!(tdt.packet_done());
                    let check = STATUS_WORD_SANITY_CHECKER.sanity_check_tdt(&tdt);
                    if check.is_err() {
                        eprintln!("TDT sanity check failed: {}", check.err().unwrap());
                        eprintln!("TDT: {:02X?}", gbt_word);
                    }
                    self.last_tdt = Some(tdt);

                    m.transition(WasTDTPacketDoneTrue).as_enum()
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(WasData).as_enum()
                }
            }
            CihwStByWasTDTpacketDoneFalse(m) => {
                let cihw = Ihw::load(&mut gbt_word.clone()).unwrap();
                cihw.print();
                let check = STATUS_WORD_SANITY_CHECKER.sanity_check_ihw(&cihw);
                if check.is_err() {
                    eprintln!("CIHW sanity check failed: {}", check.err().unwrap());
                    eprintln!("CIHW: {:02X?}", gbt_word);
                }
                self.last_ihw = Some(cihw);
                m.transition(Next).as_enum()
            }
            TdhStByWasTDTPacketDoneTrue(m) => {
                let tdh = Tdh::load(&mut gbt_word.clone()).unwrap();
                tdh.print();
                if tdh.internal_trigger() != 1 {
                    eprintln!("TDH internal trigger is not 1: {:02X?}", gbt_word);
                    tdh.print();
                    self.last_tdh.as_ref().unwrap().print();
                    RdhCRUv7::print_header_text();
                    self.last_rdh.as_ref().unwrap().print();
                }
                debug_assert!(tdh.internal_trigger() == 1);
                let check = STATUS_WORD_SANITY_CHECKER.sanity_check_tdh(&tdh);
                if check.is_err() {
                    eprintln!("TDH sanity check failed: {}", check.err().unwrap());
                    eprintln!("TDH: {:02X?}", gbt_word);
                    eprintln!("State is now: {:?}", m);
                }
                self.last_tdh = Some(tdh);
                match self.last_tdh.as_ref().unwrap().no_data() {
                    0 => m.transition(NoDataFalse).as_enum(),
                    1 => m.transition(NoDataTrue).as_enum(),
                    _ => unreachable!(),
                }
            }
            Ddw0StByWasTDTPacketDoneTrue(m) => {
                let ddw0 = Ddw0::load(&mut gbt_word.clone()).unwrap();
                ddw0.print();
                debug_assert!(rdh.rdh2.stop_bit == 1);
                debug_assert!(rdh.rdh2.pages_counter > 0);
                let check = STATUS_WORD_SANITY_CHECKER.sanity_check_ddw0(&ddw0);
                if check.is_err() {
                    eprintln!("DDW0 sanity check failed: {}", check.err().unwrap());
                    eprintln!("DDW0: {:02X?}", gbt_word);
                }
                self.last_ddw0 = Some(ddw0);
                m.transition(Next).as_enum()
            }
            Ddw0StByWasTDTandHBa(m) => {
                let ddw0 = Ddw0::load(&mut gbt_word.clone()).unwrap();
                ddw0.print();
                if rdh.rdh2.stop_bit == 0 {
                    rdh.rdh2.print();
                    eprint!("RDH2: {:?} ", rdh.rdh2);
                }
                debug_assert!(rdh.rdh2.stop_bit == 1);
                debug_assert!(rdh.rdh2.pages_counter > 0);
                let trigger_type = rdh.rdh2.trigger_type;
                debug_assert!((trigger_type & 0b10) == 0b10);
                let check = STATUS_WORD_SANITY_CHECKER.sanity_check_ddw0(&ddw0);
                if check.is_err() {
                    eprintln!("DDW0 sanity check failed: {}", check.err().unwrap());
                    eprintln!("DDW0: {:02X?}", gbt_word);
                }
                self.last_ddw0 = Some(ddw0);
                m.transition(Next).as_enum()
            }
            DataStByWasTdh(m) => {
                if gbt_word[9] == 0xF0 {
                    // TDT ID
                    let tdt = Tdt::load(&mut gbt_word.clone()).unwrap();
                    tdt.print();
                    let check = STATUS_WORD_SANITY_CHECKER.sanity_check_tdt(&tdt);
                    if check.is_err() {
                        eprintln!("TDT sanity check failed: {}", check.err().unwrap());
                        eprintln!("TDT: {:02X?}", gbt_word);
                    }
                    self.last_tdt = Some(tdt);
                    m.transition(WasTDTPacketDoneTrue).as_enum()
                } else {
                    // TODO: Check if the data identifier is valid
                    m.transition(WasData).as_enum()
                }
            }
            Ddw0StByWasDdw0(m) => {
                let ddw0 = Ddw0::load(&mut gbt_word.clone()).unwrap();
                ddw0.print();
                debug_assert!(rdh.rdh2.stop_bit == 1);
                debug_assert!(rdh.rdh2.pages_counter >= 1);

                let check = STATUS_WORD_SANITY_CHECKER.sanity_check_ddw0(&ddw0);
                if check.is_err() {
                    eprintln!("DDW0 sanity check failed: {}", check.err().unwrap());
                    eprintln!("DDW0: {:02X?}", gbt_word);
                }
                self.last_ddw0 = Some(ddw0);
                m.transition(Next).as_enum()
            }
        };

        self.sm = nxt_st;

        Ok(())
    }
}
