#![allow(dead_code)]
//! Word definitions and utility functions for working with ALPIDE data words

use super::{
    data_words::{ib_data_word_id_to_lane, ob_data_word_id_to_lane},
    Layer,
};

/// Struct for storing the contents of data words for a specific lane, in a readout frame
#[derive(Default)]
pub struct LaneDataFrame {
    /// The lane ID
    pub(crate) lane_id: u8,
    /// The data contents of data words (the 9 bytes of data excluding the ID)
    pub(crate) lane_data: Vec<u8>,
}

impl LaneDataFrame {
    /// Returns the lane number for the [LaneDataFrame] based on the [Layer] it is from
    ///
    /// The [LaneDataFrame] does not store the barrel it is from, so this must be provided.
    pub fn lane_number(&self, from_barrel: Layer) -> u8 {
        match from_barrel {
            Layer::Inner => ib_data_word_id_to_lane(self.lane_id),
            Layer::Middle | Layer::Outer => ob_data_word_id_to_lane(self.lane_id),
        }
    }
}

/// All the possible words that can be found in the ALPIDE data stream
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AlpideWord {
    ChipHeader,     // 1010<chip id[3:0]><BUNCH_COUNTER_FOR_FRAME[10:3]>
    ChipEmptyFrame, // 1110<chip id[3:0]><BUNCH COUNTER FOR FRAME[10:3]>
    ChipTrailer,    // 1011<readout flags[3:0]>
    RegionHeader,   // 110<region id[4:0]>
    DataShort,      // 01<encoder id[3:0]><addr[9:0]>
    DataLong,       // 00<encoder id[3:0]><addr[9:0]>0<hit map[6:0]>
    BusyOn,         // 1111_0001
    BusyOff,        // 1111_0000
}

impl AlpideWord {
    const CHIP_HEADER: u8 = 0xA0; // 1010_<chip_id[3:0]> next 8 bits are bit [10:3] of the bunch counter for the frame
    const CHIP_EMPTY_FRAME: u8 = 0xE0; // 1110_<chip_id[3:0]> next 8 bits are bit [10:3] of the bunch counter for the frame
    const CHIP_TRAILER: u8 = 0xB0; // 1011_<readout_flags[3:0]>
    const REGION_HEADER: u8 = 0xC0; // 110<region_id[4:0]>
    const DATA_SHORT: u8 = 0b0100_0000; // 01<encoder_id[3:0]> next 10 bits are <addr[9:0]>
    const DATA_LONG: u8 = 0b0000_0000; // 00<encoder_id[3:0]> next 18 bits are <addr[9:0]>_0_<hit_map[6:0]>
    const BUSY_ON: u8 = 0xF0;
    const BUSY_OFF: u8 = 0xF1;
    // ALPIDE Protocol Extension (APE) words
    const APE_PADDING: u8 = 0x00;
    const APE_STRIP_START: u8 = 0xF2; // Lane status = WARNING
    const APE_DET_TIMEOUT: u8 = 0xF4; // Lane status = FATAL
    const APE_OOT: u8 = 0xF5; // Lane status = FATAL
    const APE_PROTOCOL_ERROR: u8 = 0xF6; // Lane status = FATAL
    const APE_LANE_FIFO_OVERFLOW_ERROR: u8 = 0xF7; // Lane status = FATAL
    const APE_FSM_ERROR: u8 = 0xF8; // Lane status = FATAL
    const APE_PENDING_DETECTOR_EVENT_LIMIT: u8 = 0xF9; // Lane status = FATAL
    const APE_PENDING_LANE_EVENT_LIMIT: u8 = 0xFA; // Lane status = FATAL
    const APE_O2N_ERROR: u8 = 0xFB; // Lane status = FATAL
    const APE_RATE_MISSING_TRG_ERROR: u8 = 0xFC; // Lane status = FATAL
    const APE_PE_DATA_MISSING: u8 = 0xFD; // Lane status = WARNING
    const APE_OOT_DATA_MISSING: u8 = 0xFE; // Lane status = WARNING
    pub fn from_byte(b: u8) -> Result<AlpideWord, ()> {
        match b {
            // Exact matches
            Self::BUSY_ON => Ok(AlpideWord::BusyOn),
            Self::BUSY_OFF => Ok(AlpideWord::BusyOff),
            four_msb => match four_msb & 0xF0 {
                // Match on the 4 MSB
                Self::CHIP_HEADER => Ok(AlpideWord::ChipHeader),
                Self::CHIP_EMPTY_FRAME => Ok(AlpideWord::ChipEmptyFrame),
                Self::CHIP_TRAILER => Ok(AlpideWord::ChipTrailer),
                three_msb => match three_msb & 0xE0 {
                    // Match on the 3 MSB
                    Self::REGION_HEADER => Ok(AlpideWord::RegionHeader),
                    two_msb => match two_msb & 0xC0 {
                        // Match on the 2 MSB
                        Self::DATA_SHORT => Ok(AlpideWord::DataShort),
                        Self::DATA_LONG => Ok(AlpideWord::DataLong),
                        _ => Err(()),
                    },
                },
            },
        }
    }
}

/// Contains information from a single ALPIDE chip in a single frame
///
/// Unsafe if used outside of the context of a single frame
pub struct AlpideFrameChipData {
    /// The ID of the chip the data is from
    pub(crate) chip_id: u8,
    /// Bunch counter for the frame \[10:3\]
    pub(crate) bunch_counter: Option<u8>,
    /// Other data from the chip
    pub(crate) data: Vec<u8>,
}

impl AlpideFrameChipData {
    /// Create a new instance from the chip ID
    pub fn from_id(chip_id: u8) -> Self {
        Self {
            chip_id,
            bunch_counter: None,
            data: Vec::new(),
        }
    }
    /// Create a new instance from the chip ID but disallow adding data
    ///
    /// A light weight version of `from_id` that is used when the data is not needed
    pub fn from_id_no_data(chip_id: u8) -> Self {
        Self {
            chip_id,
            bunch_counter: None,
            data: Vec::with_capacity(0),
        }
    }

    /// Store the bunch counter for a chip in a frame
    ///
    /// If the bunch counter has already been set, an error is returned describing the Chip ID,
    /// the current bunch counter, and the bunch counter that was attempted to be set
    pub fn store_bc(&mut self, bc: u8) -> Result<(), String> {
        if self.bunch_counter.is_some() {
            return Err(format!(
                "Bunch counter already set for chip {id}, is {current_bc}, tried to set to {new_bc}",
                id = self.chip_id,
                current_bc = self.bunch_counter.unwrap(),
                new_bc = bc
            ));
        }
        self.bunch_counter = Some(bc);
        Ok(())
    }
}
