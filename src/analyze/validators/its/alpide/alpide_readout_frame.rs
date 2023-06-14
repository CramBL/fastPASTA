//! Contains the [AlpideReadoutFrame] struct that stores information about a readout frame from the ALPIDE chips.
//!
//! A readout frame should contain data from multiple lanes, and the data from each lane is stored in a [LaneDataFrame].
use crate::words::its::{alpide_words::LaneDataFrame, data_words::ib_data_word_id_to_lane, Layer};

/// Struct for storing the contents of a single ALPIDE readout frame
#[derive(Default)]
pub struct AlpideReadoutFrame {
    pub(crate) frame_start_mem_pos: u64,
    pub(crate) frame_end_mem_pos: u64,
    pub(crate) lane_data_frames: Vec<LaneDataFrame>, // Vector of data frames for each lane
    from_layer: Option<Layer>,
}

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

    /// Returns the barrel that the readout frame is from
    pub fn is_from_layer(&self) -> Layer {
        self.from_layer.expect("No barrel set for readout frame")
    }

    /// Check if the frame is valid in terms of number of lanes in the data and for IB, the lane grouping.
    pub fn check_frame_lanes_valid(&self) -> Result<(), String> {
        let expect_lane_count = match self.is_from_layer() {
            Layer::Inner => Self::IL_FRAME_LANE_COUNT,
            Layer::Middle => Self::ML_FRAME_LANE_COUNT,
            Layer::Outer => Self::OL_FRAME_LANE_COUNT,
        };

        // Check number of lanes is correct, then if IB, also check lane grouping is correct
        if self.lane_data_frames.len() != expect_lane_count {
            Err(format!(
                "Invalid number of lanes: {num_lanes}, expected {expect_lane_count}",
                num_lanes = self.lane_data_frames.len()
            ))
        } else if self.is_from_layer() == Layer::Inner {
            // Check frame lane grouping is correct (these groupings are hardcoded in the firmware)
            let mut lane_ids = self
                .lane_data_frames
                .iter()
                .map(|lane_data_frame| ib_data_word_id_to_lane(lane_data_frame.lane_id))
                .collect::<Vec<u8>>();
            lane_ids.sort();
            match lane_ids.as_slice() {
                &[0, 1, 2] | &[3, 4, 5] | &[6, 7, 8] => return Ok(()),
                _ => return Err("Invalid lane grouping".to_string()),
            }
        } else {
            // No grouping to check for outer barrel
            return Ok(());
        }
    }
}
