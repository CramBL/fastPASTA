#![allow(non_camel_case_types)] // An exception to the Rust naming convention, for the state machine macro types

pub enum PayloadWord {
    IHW,
    IHW_continuation,
    TDH,
    TDH_continuation,
    TDT,
    CDW,
    DataWord,
    DDW0,
}

use sm::sm;
sm! {
    // All states have the '_' suffix and events have '_' prefix so they show up as `STATE_BY_EVENT` in the generated code
    // The statemachine macro notation goes like this:
    // EventName { StateFrom => StateTo }
    ITS_Payload_Continuous {

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

use self::ITS_Payload_Continuous::IHW_;
pub struct ItsPayloadFsmContinuous {
    state_machine: ITS_Payload_Continuous::Variant,
}

impl Default for ItsPayloadFsmContinuous {
    fn default() -> Self {
        Self::new()
    }
}

impl ItsPayloadFsmContinuous {
    pub fn new() -> Self {
        Self {
            state_machine: ITS_Payload_Continuous::Machine::new(IHW_).as_enum(),
        }
    }

    pub fn reset_fsm(&mut self) {
        self.state_machine = ITS_Payload_Continuous::Machine::new(IHW_).as_enum();
    }

    pub fn advance(&mut self, gbt_word: &[u8]) -> PayloadWord {
        use crate::words::status_words;
        use ITS_Payload_Continuous::Variant::*;
        use ITS_Payload_Continuous::*;

        let current_state = self.state_machine.clone();

        let (next_state, current_word): (Variant, PayloadWord) = match current_state {
            InitialIHW_(m) => (m.transition(_WasIhw).as_enum(), PayloadWord::IHW),

            TDH_By_WasIhw(m) => match status_words::util::tdh_no_data(gbt_word) {
                false => (m.transition(_NoDataFalse).as_enum(), PayloadWord::TDH),
                true => (m.transition(_NoDataTrue).as_enum(), PayloadWord::TDH),
            },

            DDW0_or_TDH_By_NoDataTrue(m) => match gbt_word[9] {
                0xE8 => (m.transition(_NoDataFalse).as_enum(), PayloadWord::TDH),
                _ => {
                    debug_assert!(gbt_word[9] == 0xE4);
                    (m.transition(_WasDdw0).as_enum(), PayloadWord::DDW0)
                }
            },

            DATA_By_NoDataFalse(m) => match gbt_word[9] {
                0xF0 if status_words::util::tdt_packet_done(gbt_word) => (
                    m.transition(_WasTDTpacketDoneTrue).as_enum(),
                    PayloadWord::TDT,
                ),
                0xF0 if !status_words::util::tdt_packet_done(gbt_word) => (
                    m.transition(_WasTDTpacketDoneFalse).as_enum(),
                    PayloadWord::TDT,
                ),
                0xF8 => (m.transition(_WasData).as_enum(), PayloadWord::CDW),
                _ => (m.transition(_WasData).as_enum(), PayloadWord::DataWord),
            },

            DATA_By_WasData(m) => match gbt_word[9] {
                0xF0 if status_words::util::tdt_packet_done(gbt_word) => (
                    m.transition(_WasTDTpacketDoneTrue).as_enum(),
                    PayloadWord::TDT,
                ),
                0xF0 if !status_words::util::tdt_packet_done(gbt_word) => (
                    m.transition(_WasTDTpacketDoneFalse).as_enum(),
                    PayloadWord::TDT,
                ),
                0xF8 => (m.transition(_WasData).as_enum(), PayloadWord::CDW),
                _ => (m.transition(_WasData).as_enum(), PayloadWord::DataWord),
            },

            c_IHW_By_WasTDTpacketDoneFalse(m) => {
                (m.transition(_Next).as_enum(), PayloadWord::IHW_continuation)
            }

            c_TDH_By_Next(m) => (m.transition(_Next).as_enum(), PayloadWord::TDH_continuation),

            c_DATA_By_Next(m) => match gbt_word[9] {
                0xF0 if status_words::util::tdt_packet_done(gbt_word) => (
                    m.transition(_WasTDTpacketDoneTrue).as_enum(),
                    PayloadWord::TDT,
                ),
                0xF0 if !status_words::util::tdt_packet_done(gbt_word) => (
                    m.transition(_WasTDTpacketDoneFalse).as_enum(),
                    PayloadWord::TDT,
                ),
                0xF8 => (m.transition(_WasData).as_enum(), PayloadWord::CDW),
                _ => (m.transition(_WasData).as_enum(), PayloadWord::DataWord),
            },

            c_DATA_By_WasData(m) => match gbt_word[9] {
                0xF0 if status_words::util::tdt_packet_done(gbt_word) => (
                    m.transition(_WasTDTpacketDoneTrue).as_enum(),
                    PayloadWord::TDT,
                ),
                0xF0 if !status_words::util::tdt_packet_done(gbt_word) => (
                    m.transition(_WasTDTpacketDoneFalse).as_enum(),
                    PayloadWord::TDT,
                ),
                0xF8 => (m.transition(_WasData).as_enum(), PayloadWord::CDW),
                _ => (m.transition(_WasData).as_enum(), PayloadWord::DataWord),
            },

            DDW0_or_TDH_or_IHW_By_WasTDTpacketDoneTrue(m) => match gbt_word[9] {
                0xE8 if status_words::util::tdh_no_data(gbt_word) => {
                    (m.transition(_NoDataTrue).as_enum(), PayloadWord::TDH)
                }
                0xE8 if !status_words::util::tdh_no_data(gbt_word) => {
                    (m.transition(_NoDataFalse).as_enum(), PayloadWord::TDH)
                }
                0xE4 => (m.transition(_WasDdw0).as_enum(), PayloadWord::DDW0),
                _ => (m.transition(_WasIhw).as_enum(), PayloadWord::IHW),
            },
            IHW_By_WasDdw0(m) => (m.transition(_WasIhw).as_enum(), PayloadWord::IHW),
        };

        self.state_machine = next_state;
        current_word
    }
}
