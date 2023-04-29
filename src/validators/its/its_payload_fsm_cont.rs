//! State machine for ITS payload continuous mode
#![allow(non_camel_case_types)] // An exception to the Rust naming convention, for the state machine macro types

/// Payload word types
pub enum PayloadWord {
    /// ITS Header Word
    IHW,
    /// ITS Header Word in continuation mode
    IHW_continuation,
    /// Trigger Data Header
    TDH,
    /// Trigger Data Header in continuation mode
    TDH_continuation,
    /// Trigger Data Header succeeding a TDT with packet done flag set
    TDH_after_packet_done,
    /// Trigger Data Trailer
    TDT,
    /// Calibration Data Word
    CDW,
    /// Data
    DataWord,
    /// Diagnostic Data Word 0
    DDW0,
}

/// Types that can be returned as an error from the FSM.
///
/// The first word indicate the best guess made by the FSM
/// e.g. TDH_or_DDW0 could be either TDH or DDW0 but the best guess is TDH
/// The best guess is made based on heurestics.
/// Such as the perceived most likely candidate that could have an ID error,
/// or a word that would be easier to get the FSM back on track based on.
pub enum AmbigiousError {
    /// ID error when both DDW0 or TDH would be a valid word.
    TDH_or_DDW0,
    /// ID error when a Data word, TDT, or CDW could be valid words.
    DW_or_TDT_CDW,
    /// ID error when a DDW0, TDH, or IHW could be valid words.
    DDW0_or_TDH_IHW,
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
/// State machine for ITS payload continuous mode.
pub struct ItsPayloadFsmContinuous {
    state_machine: ITS_Payload_Continuous::Variant,
}

impl Default for ItsPayloadFsmContinuous {
    fn default() -> Self {
        Self::new()
    }
}

impl ItsPayloadFsmContinuous {
    /// Create a new state machine in the initial state.
    pub fn new() -> Self {
        Self {
            state_machine: ITS_Payload_Continuous::Machine::new(IHW_).as_enum(),
        }
    }
    /// Reset the state machine to the initial state.
    pub fn reset_fsm(&mut self) {
        self.state_machine = ITS_Payload_Continuous::Machine::new(IHW_).as_enum();
    }

    /// Advance the state machine by one word.
    ///
    /// Takes a slice of 10 bytes representing the GBT word.
    /// Returns the type of the word wrapped in an OK if the ID was identified.
    /// If the ID could not be determined, a best guess is returned wrapped in an error type to be handled by the caller
    pub fn advance(&mut self, gbt_word: &[u8]) -> Result<PayloadWord, AmbigiousError> {
        use crate::words::its::status_words;
        use ITS_Payload_Continuous::Variant::*;
        use ITS_Payload_Continuous::*;

        let current_state = self.state_machine.clone();

        let (next_state, current_word): (Variant, Result<PayloadWord, AmbigiousError>) =
            match current_state {
                InitialIHW_(m) => (m.transition(_WasIhw).as_enum(), Ok(PayloadWord::IHW)),

                TDH_By_WasIhw(m) => match status_words::util::tdh_no_data(gbt_word) {
                    false => (m.transition(_NoDataFalse).as_enum(), Ok(PayloadWord::TDH)),
                    true => (m.transition(_NoDataTrue).as_enum(), Ok(PayloadWord::TDH)),
                },

                DDW0_or_TDH_By_NoDataTrue(m) => match gbt_word[9] {
                    0xE8 => (m.transition(_NoDataFalse).as_enum(), Ok(PayloadWord::TDH)),
                    0xE4 => (m.transition(_WasDdw0).as_enum(), Ok(PayloadWord::DDW0)),
                    // Error in ID, assuming TDH to try to stay on track, but return as error to be handled by caller
                    _ => (
                        m.transition(_NoDataFalse).as_enum(),
                        Err(AmbigiousError::TDH_or_DDW0),
                    ),
                },

                DATA_By_NoDataFalse(m) => match gbt_word[9] {
                    0xF0 if status_words::util::tdt_packet_done(gbt_word) => (
                        m.transition(_WasTDTpacketDoneTrue).as_enum(),
                        Ok(PayloadWord::TDT),
                    ),
                    0xF0 if !status_words::util::tdt_packet_done(gbt_word) => (
                        m.transition(_WasTDTpacketDoneFalse).as_enum(),
                        Ok(PayloadWord::TDT),
                    ),
                    0xF8 => (m.transition(_WasData).as_enum(), Ok(PayloadWord::CDW)),
                    // All ranges that are legal data word IDs
                    0x20..=0x28 | 0x40..=0x46 | 0x48..=0x4E | 0x50..=0x56 | 0x58..=0x5E => {
                        (m.transition(_WasData).as_enum(), Ok(PayloadWord::DataWord))
                    }
                    // Assume data word but return as error
                    _ => (
                        m.transition(_WasData).as_enum(),
                        Err(AmbigiousError::DW_or_TDT_CDW),
                    ),
                },

                DATA_By_WasData(m) => match gbt_word[9] {
                    0xF0 if status_words::util::tdt_packet_done(gbt_word) => (
                        m.transition(_WasTDTpacketDoneTrue).as_enum(),
                        Ok(PayloadWord::TDT),
                    ),
                    0xF0 if !status_words::util::tdt_packet_done(gbt_word) => (
                        m.transition(_WasTDTpacketDoneFalse).as_enum(),
                        Ok(PayloadWord::TDT),
                    ),
                    0xF8 => (m.transition(_WasData).as_enum(), Ok(PayloadWord::CDW)),
                    // All ranges that are legal data word IDs
                    0x20..=0x28 | 0x40..=0x46 | 0x48..=0x4E | 0x50..=0x56 | 0x58..=0x5E => {
                        (m.transition(_WasData).as_enum(), Ok(PayloadWord::DataWord))
                    }
                    // Assume data word but return as error
                    _ => (
                        m.transition(_WasData).as_enum(),
                        Err(AmbigiousError::DW_or_TDT_CDW),
                    ),
                },

                c_IHW_By_WasTDTpacketDoneFalse(m) => (
                    m.transition(_Next).as_enum(),
                    Ok(PayloadWord::IHW_continuation),
                ),

                c_TDH_By_Next(m) => (
                    m.transition(_Next).as_enum(),
                    Ok(PayloadWord::TDH_continuation),
                ),

                c_DATA_By_Next(m) => match gbt_word[9] {
                    0xF0 if status_words::util::tdt_packet_done(gbt_word) => (
                        m.transition(_WasTDTpacketDoneTrue).as_enum(),
                        Ok(PayloadWord::TDT),
                    ),
                    0xF0 if !status_words::util::tdt_packet_done(gbt_word) => (
                        m.transition(_WasTDTpacketDoneFalse).as_enum(),
                        Ok(PayloadWord::TDT),
                    ),
                    0xF8 => (m.transition(_WasData).as_enum(), Ok(PayloadWord::CDW)),
                    // All ranges that are legal data word IDs
                    0x20..=0x28 | 0x40..=0x46 | 0x48..=0x4E | 0x50..=0x56 | 0x58..=0x5E => {
                        (m.transition(_WasData).as_enum(), Ok(PayloadWord::DataWord))
                    }
                    // Assume data word but return as error
                    _ => (
                        m.transition(_WasData).as_enum(),
                        Err(AmbigiousError::DW_or_TDT_CDW),
                    ),
                },

                c_DATA_By_WasData(m) => match gbt_word[9] {
                    0xF0 if status_words::util::tdt_packet_done(gbt_word) => (
                        m.transition(_WasTDTpacketDoneTrue).as_enum(),
                        Ok(PayloadWord::TDT),
                    ),
                    0xF0 if !status_words::util::tdt_packet_done(gbt_word) => (
                        m.transition(_WasTDTpacketDoneFalse).as_enum(),
                        Ok(PayloadWord::TDT),
                    ),
                    0xF8 => (m.transition(_WasData).as_enum(), Ok(PayloadWord::CDW)),
                    // All ranges that are legal data word IDs
                    0x20..=0x28 | 0x40..=0x46 | 0x48..=0x4E | 0x50..=0x56 | 0x58..=0x5E => {
                        (m.transition(_WasData).as_enum(), Ok(PayloadWord::DataWord))
                    }
                    // Assume data word but return as error
                    _ => (
                        m.transition(_WasData).as_enum(),
                        Err(AmbigiousError::DW_or_TDT_CDW),
                    ),
                },

                DDW0_or_TDH_or_IHW_By_WasTDTpacketDoneTrue(m) => match gbt_word[9] {
                    0xE8 if status_words::util::tdh_no_data(gbt_word) => (
                        m.transition(_NoDataTrue).as_enum(),
                        Ok(PayloadWord::TDH_after_packet_done),
                    ),
                    0xE8 if !status_words::util::tdh_no_data(gbt_word) => (
                        m.transition(_NoDataFalse).as_enum(),
                        Ok(PayloadWord::TDH_after_packet_done),
                    ),
                    0xE4 => (m.transition(_WasDdw0).as_enum(), Ok(PayloadWord::DDW0)),
                    0xE0 => (m.transition(_WasIhw).as_enum(), Ok(PayloadWord::IHW)),
                    // Assume DDW0 but return as error
                    _ => (
                        m.transition(_WasDdw0).as_enum(),
                        Err(AmbigiousError::DDW0_or_TDH_IHW),
                    ),
                },
                IHW_By_WasDdw0(m) => (m.transition(_WasIhw).as_enum(), Ok(PayloadWord::IHW)),
            };

        self.state_machine = next_state;
        current_word
    }
}
