//! Contains the [AlpideReadoutFrame] struct that stores information about a readout frame from the ALPIDE chips.
//!
//! A readout frame should contain data from multiple lanes, and the data from each lane is stored in a [LaneDataFrame].
use crate::words::its::{alpide_words::LaneDataFrame, data_words::ib_data_word_id_to_lane, Layer};

/// Struct for storing the contents of a single ALPIDE readout frame
#[derive(Default)]
pub struct AlpideReadoutFrame {
    frame_start_mem_pos: u64,
    frame_end_mem_pos: u64,
    lane_data_frames: Vec<LaneDataFrame>, // Vector of data frames for each lane
    from_layer: Option<Layer>,
}

// impl for core functionality
impl AlpideReadoutFrame {
    const IL_FRAME_LANE_COUNT: usize = 3;
    const ML_FRAME_LANE_COUNT: usize = 8;
    const OL_FRAME_LANE_COUNT: usize = 14;
    /// Create a new ALPIDE readout frame from the given memory position.
    pub fn new(start_mem_pos: u64) -> Self {
        Self {
            frame_start_mem_pos: start_mem_pos,
            ..Default::default()
        }
    }

    /// Stores the 9 data bytes from an ITS data word byte data slice (does not store the ID byte more than once) by appending it to the lane data.
    pub fn store_lane_data(&mut self, data_word: &[u8], from_layer: Layer) {
        debug_assert_eq!(
            self.frame_end_mem_pos, 0,
            "Attempted to store lane data after the data frame was closed"
        );
        if self.from_layer.is_none() {
            self.from_layer = Some(from_layer);
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

    /// Check if the frame is valid in terms of number of lanes in the data and for IB, the lane grouping.
    pub fn check_frame_lanes_valid(&self, fatal_lanes: Option<&[u8]>) -> Result<(), String> {
        debug_assert_ne!(
            self.frame_end_mem_pos, 0,
            "Attempted check a lane data frame's validity before closing it"
        );
        let mut expect_lane_count = match self.from_layer() {
            Layer::Inner => Self::IL_FRAME_LANE_COUNT,
            Layer::Middle => Self::ML_FRAME_LANE_COUNT,
            Layer::Outer => Self::OL_FRAME_LANE_COUNT,
        };

        // Check if any lanes are in FATAL state, in that case lower the expected lane count
        if let Some(fatal_lanes) = fatal_lanes {
            expect_lane_count -= fatal_lanes.len();
        }

        // Check number of lanes is correct, then if IB, also check lane grouping is correct
        if self.lane_data_frames.len() != expect_lane_count {
            Err(format!(
                "Invalid number of lanes: {num_lanes}, expected {expect_lane_count}",
                num_lanes = self.lane_data_frames.len()
            ))
        } else if self.from_layer() == Layer::Inner {
            // Check frame lane grouping is correct (these groupings are hardcoded in the firmware)
            validate_inner_lane_groupings(&self.lane_data_frames, fatal_lanes)
        } else {
            // No grouping to check for outer barrel
            return Ok(());
        }
    }
}

/// Validate the inner lane grouping (these groupings are hardcoded in the firmware)
/// if no fatal lanes are present, then the valid groupings are:
///  - `[0, 1, 2]`
///  - `[3, 4, 5]`
///  - `[6, 7, 8]`
///
/// if there's any known fatal lanes then the groupings are adjusted to exclude those lanes.
/// e.g. if lane 1 is fatal then the first valid grouping is `[0, 2]` instead of `[0, 1, 2]`
#[inline]
pub(crate) fn validate_inner_lane_groupings(
    lane_data_frames: &[LaneDataFrame],
    fatal_lanes: Option<&[u8]>,
) -> Result<(), String> {
    let mut lane_ids = lane_data_frames
        .iter()
        .map(|lane_data_frame| ib_data_word_id_to_lane(lane_data_frame.lane_id))
        .collect::<Vec<u8>>();
    lane_ids.sort_unstable();

    let mut valid_lane_groupings: [Vec<u8>; 3] = [vec![0, 1, 2], vec![3, 4, 5], vec![6, 7, 8]];
    if let Some(fatal_lane) = fatal_lanes {
        for fl in fatal_lane {
            match fl {
                0..=2 => unsafe {
                    valid_lane_groupings
                        .get_unchecked_mut(0)
                        .retain(|&x| x != *fl)
                },
                3..=5 => unsafe {
                    valid_lane_groupings
                        .get_unchecked_mut(1)
                        .retain(|&x| x != *fl)
                },
                6..=8 => unsafe {
                    valid_lane_groupings
                        .get_unchecked_mut(2)
                        .retain(|&x| x != *fl)
                },
                _ => unreachable!("Invalid fatal lane number: {fl}"),
            }
        }
    }
    for lane_grouping in valid_lane_groupings.iter() {
        if lane_ids == lane_grouping.as_slice() {
            return Ok(());
        }
    }
    Err("Invalid lane grouping".into())
}

// impl for simple utility functions
impl AlpideReadoutFrame {
    /// Returns the [Layer] that the readout frame is from
    pub fn from_layer(&self) -> Layer {
        self.from_layer.expect("No barrel set for readout frame")
    }

    /// Close an [AlpideReadoutFrame] by setting the memory position where it ends
    pub fn close_frame(&mut self, frame_end_mem_pos: u64) {
        debug_assert_eq!(
            self.frame_end_mem_pos, 0,
            "frame_end_mem_pos set more than once!"
        );
        self.frame_end_mem_pos = frame_end_mem_pos
    }

    /// Get the memory position where the [AlpideReadoutFrame] started
    pub fn start_mem_pos(&self) -> u64 {
        self.frame_start_mem_pos
    }

    /// Get the memory position where the [AlpideReadoutFrame] ended
    pub fn end_mem_pos(&self) -> u64 {
        self.frame_end_mem_pos
    }

    /// Get the number of [LaneDataFrame]s in the [AlpideReadoutFrame]
    ///
    /// An empty readout frame after a TDT with packet done indicates a readout error and should be reported.
    pub fn is_empty(&self) -> bool {
        self.lane_data_frames.is_empty()
    }

    /// Borrow the [LaneDataFrame]s as a slice
    pub fn lane_data_frames_as_slice(&self) -> &[LaneDataFrame] {
        &self.lane_data_frames
    }

    /// Drain the vector of [LaneDataFrame]s
    pub fn drain_lane_data_frames(&mut self) -> std::vec::Drain<LaneDataFrame> {
        self.lane_data_frames.drain(..)
    }

    /// Take (consumes) the vector of [LaneDataFrame]s
    pub fn take_lane_data_frames(&mut self) -> std::vec::Vec<LaneDataFrame> {
        std::mem::take(&mut self.lane_data_frames)
    }
}
