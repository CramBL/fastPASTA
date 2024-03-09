//! State machine for ITS payload continuous mode
#![allow(non_camel_case_types)]

use self::ITS_Payload_Continuous::IHW_;
use crate::util::*;

/// Types that can be returned as an error from the FSM.
///
/// The first word indicate the best guess made by the FSM
/// e.g. TDH_or_DDW0 could be either TDH or DDW0 but the best guess is TDH
/// The best guess is made based on heurestics.
/// Such as the perceived most likely candidate that could have an ID error,
/// or a word that would be easier to get the FSM back on track based on.
#[derive(Debug, Clone, Copy)]
pub enum AmbigiousError {
    /// ID error when both DDW0 or TDH would be a valid word.
    TDH_or_DDW0,
    /// ID error when a Data word, TDT, or CDW could be valid words.
    DW_or_TDT_CDW,
    /// ID error when a DDW0, TDH, or IHW could be valid words.
    DDW0_or_TDH_IHW,
}

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
            TDH_ => DDW0_or_TDH_or_IHW_,
            DDW0_or_TDH_ => DDW0_or_TDH_or_IHW_,
            DDW0_or_TDH_or_IHW_ => DDW0_or_TDH_or_IHW_
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
    pub fn advance(&mut self, gbt_word: &[u8]) -> Result<ItsPayloadWord, AmbigiousError> {
        use ITS_Payload_Continuous as event;
        use ITS_Payload_Continuous::Variant as state;

        let current_state = self.state_machine.clone();

        let (next_state, current_word): (state, Result<ItsPayloadWord, AmbigiousError>) =
            match current_state {
                state::DATA_By_WasData(stm) => match gbt_word[9] {
                    // All ranges that are legal data word IDs
                    0x20..=0x28 | 0x40..=0x46 | 0x48..=0x4E | 0x50..=0x56 | 0x58..=0x5E => (
                        stm.transition(event::_WasData).as_enum(),
                        Ok(ItsPayloadWord::DataWord),
                    ),
                    0xF0 if !tdt_packet_done(gbt_word) => (
                        stm.transition(event::_WasTDTpacketDoneFalse).as_enum(),
                        Ok(ItsPayloadWord::TDT),
                    ),
                    0xF0 if tdt_packet_done(gbt_word) => (
                        stm.transition(event::_WasTDTpacketDoneTrue).as_enum(),
                        Ok(ItsPayloadWord::TDT),
                    ),
                    0xF8 => (
                        stm.transition(event::_WasData).as_enum(),
                        Ok(ItsPayloadWord::CDW),
                    ),
                    // Assume data word but return as error
                    _ => (
                        stm.transition(event::_WasData).as_enum(),
                        Err(AmbigiousError::DW_or_TDT_CDW),
                    ),
                },

                state::DDW0_or_TDH_or_IHW_By_NoDataTrue(stm) => match gbt_word[9] {
                    0xE8 if tdh_no_data(gbt_word) => (
                        stm.transition(event::_NoDataTrue).as_enum(),
                        Ok(ItsPayloadWord::TDH_after_packet_done),
                    ),
                    0xE8 if !tdh_no_data(gbt_word) => (
                        stm.transition(event::_NoDataFalse).as_enum(),
                        Ok(ItsPayloadWord::TDH_after_packet_done),
                    ),
                    0xE0 => (
                        stm.transition(event::_WasIhw).as_enum(),
                        Ok(ItsPayloadWord::IHW),
                    ),
                    0xE4 => (
                        stm.transition(event::_WasDdw0).as_enum(),
                        Ok(ItsPayloadWord::DDW0),
                    ),
                    // Error in ID, assuming TDH to try to stay on track, but return as error to be handled by caller
                    _ => (
                        stm.transition(event::_NoDataFalse).as_enum(),
                        Err(AmbigiousError::TDH_or_DDW0),
                    ),
                },

                state::DATA_By_NoDataFalse(stm) => match gbt_word[9] {
                    // All ranges that are legal data word IDs
                    0x20..=0x28 | 0x40..=0x46 | 0x48..=0x4E | 0x50..=0x56 | 0x58..=0x5E => (
                        stm.transition(event::_WasData).as_enum(),
                        Ok(ItsPayloadWord::DataWord),
                    ),
                    0xF0 if tdt_packet_done(gbt_word) => (
                        stm.transition(event::_WasTDTpacketDoneTrue).as_enum(),
                        Ok(ItsPayloadWord::TDT),
                    ),
                    0xF0 if !tdt_packet_done(gbt_word) => (
                        stm.transition(event::_WasTDTpacketDoneFalse).as_enum(),
                        Ok(ItsPayloadWord::TDT),
                    ),
                    0xF8 => (
                        stm.transition(event::_WasData).as_enum(),
                        Ok(ItsPayloadWord::CDW),
                    ),
                    // Assume data word but return as error
                    _ => (
                        stm.transition(event::_WasData).as_enum(),
                        Err(AmbigiousError::DW_or_TDT_CDW),
                    ),
                },

                state::TDH_By_WasIhw(stm) => (
                    if tdh_no_data(gbt_word) {
                        stm.transition(event::_NoDataTrue).as_enum()
                    } else {
                        stm.transition(event::_NoDataFalse).as_enum()
                    },
                    Ok(ItsPayloadWord::TDH),
                ),

                state::c_DATA_By_WasData(stm) => match gbt_word[9] {
                    // All ranges that are legal data word IDs
                    0x20..=0x28 | 0x40..=0x46 | 0x48..=0x4E | 0x50..=0x56 | 0x58..=0x5E => (
                        stm.transition(event::_WasData).as_enum(),
                        Ok(ItsPayloadWord::DataWord),
                    ),
                    0xF0 if tdt_packet_done(gbt_word) => (
                        stm.transition(event::_WasTDTpacketDoneTrue).as_enum(),
                        Ok(ItsPayloadWord::TDT),
                    ),
                    0xF0 if !tdt_packet_done(gbt_word) => (
                        stm.transition(event::_WasTDTpacketDoneFalse).as_enum(),
                        Ok(ItsPayloadWord::TDT),
                    ),
                    0xF8 => (
                        stm.transition(event::_WasData).as_enum(),
                        Ok(ItsPayloadWord::CDW),
                    ),
                    // Assume data word but return as error
                    _ => (
                        stm.transition(event::_WasData).as_enum(),
                        Err(AmbigiousError::DW_or_TDT_CDW),
                    ),
                },

                state::c_DATA_By_Next(stm) => match gbt_word[9] {
                    // All ranges that are legal data word IDs
                    0x20..=0x28 | 0x40..=0x46 | 0x48..=0x4E | 0x50..=0x56 | 0x58..=0x5E => (
                        stm.transition(event::_WasData).as_enum(),
                        Ok(ItsPayloadWord::DataWord),
                    ),
                    0xF0 if tdt_packet_done(gbt_word) => (
                        stm.transition(event::_WasTDTpacketDoneTrue).as_enum(),
                        Ok(ItsPayloadWord::TDT),
                    ),
                    0xF0 if !tdt_packet_done(gbt_word) => (
                        stm.transition(event::_WasTDTpacketDoneFalse).as_enum(),
                        Ok(ItsPayloadWord::TDT),
                    ),
                    0xF8 => (
                        stm.transition(event::_WasData).as_enum(),
                        Ok(ItsPayloadWord::CDW),
                    ),
                    // Assume data word but return as error
                    _ => (
                        stm.transition(event::_WasData).as_enum(),
                        Err(AmbigiousError::DW_or_TDT_CDW),
                    ),
                },

                state::c_TDH_By_Next(stm) => (
                    stm.transition(event::_Next).as_enum(),
                    Ok(ItsPayloadWord::TDH_continuation),
                ),

                state::c_IHW_By_WasTDTpacketDoneFalse(stm) => (
                    stm.transition(event::_Next).as_enum(),
                    Ok(ItsPayloadWord::IHW_continuation),
                ),

                state::DDW0_or_TDH_or_IHW_By_WasTDTpacketDoneTrue(stm) => match gbt_word[9] {
                    0xE8 if tdh_no_data(gbt_word) => (
                        stm.transition(event::_NoDataTrue).as_enum(),
                        Ok(ItsPayloadWord::TDH_after_packet_done),
                    ),
                    0xE8 if !tdh_no_data(gbt_word) => (
                        stm.transition(event::_NoDataFalse).as_enum(),
                        Ok(ItsPayloadWord::TDH_after_packet_done),
                    ),
                    0xE0 => (
                        stm.transition(event::_WasIhw).as_enum(),
                        Ok(ItsPayloadWord::IHW),
                    ),
                    0xE4 => (
                        stm.transition(event::_WasDdw0).as_enum(),
                        Ok(ItsPayloadWord::DDW0),
                    ),

                    // Assume DDW0 but return as error
                    _ => (
                        stm.transition(event::_WasDdw0).as_enum(),
                        Err(AmbigiousError::DDW0_or_TDH_IHW),
                    ),
                },
                state::IHW_By_WasDdw0(stm) => (
                    stm.transition(event::_WasIhw).as_enum(),
                    Ok(ItsPayloadWord::IHW),
                ),
                state::InitialIHW_(stm) => (
                    stm.transition(event::_WasIhw).as_enum(),
                    Ok(ItsPayloadWord::IHW),
                ),
            };

        self.state_machine = next_state;
        current_word
    }
}
