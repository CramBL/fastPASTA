#![allow(dead_code)]
//! Word definitions and utility functions for working with ALPIDE data words

/// Enum for marking if the data is from the inner or outer barrel
#[derive(Debug, PartialEq, Clone)]
pub enum Barrel {
    /// Data is from the inner barrel
    Inner,
    /// Data is from the outer barrel
    Outer,
}

/// Struct for storing the contents of a single ALPIDE readout frame
#[derive(Default)]
pub struct AlpideReadoutFrame {
    pub(crate) frame_start_mem_pos: u64,
    pub(crate) frame_end_mem_pos: u64,
    pub(crate) lane_data_frames: Vec<LaneDataFrame>,
    pub(crate) from_barrel: Option<Barrel>,
}

impl AlpideReadoutFrame {
    /// Create a new ALPIDE readout frame from the given memory position.
    pub fn new(start_mem_pos: u64) -> Self {
        Self {
            frame_start_mem_pos: start_mem_pos,
            ..Default::default()
        }
    }

    /// Stores the 9 data bytes from an ITS data word byte data slice (does not store the ID byte more than once) by appending it to the lane data.
    pub fn store_lane_data(&mut self, data_word: &[u8], from_barrel: Barrel) {
        if self.from_barrel.is_none() {
            self.from_barrel = Some(from_barrel);
        }
        match self
            .lane_data_frames
            .iter_mut()
            .find(|lane_data_frame| lane_data_frame.lane_id == data_word[9])
        {
            Some(lane_data_frame) => {
                lane_data_frame
                    .lane_data
                    .extend_from_slice(&data_word[0..=8]);
            }
            None => self.lane_data_frames.push(LaneDataFrame {
                lane_id: data_word[9],
                lane_data: data_word[0..=8].to_vec(),
            }),
        }
    }
}

/// Struct for storing the contents of data words for a specific lane, in a readout frame
#[derive(Default)]
pub struct LaneDataFrame {
    /// The lane ID
    pub(crate) lane_id: u8,
    /// The data contents of data words (the 9 bytes of data excluding the ID)
    pub(crate) lane_data: Vec<u8>,
}

/// All the possible words that can be found in the ALPIDE data stream
pub(crate) enum AlpideWord {
    ChipHeader,
    ChipEmptyFrame,
    ChipTrailer,
    RegionHeader,
    DataShort,
    DataLong,
    BusyOn,
    BusyOff,
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
                "Bunch counter already set for chip {}, is {}, tried to set to {}",
                self.chip_id,
                self.bunch_counter.unwrap(),
                bc
            ));
        }
        self.bunch_counter = Some(bc);
        Ok(())
    }
}
