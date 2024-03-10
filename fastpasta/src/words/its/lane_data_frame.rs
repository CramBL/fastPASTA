//! Struct for storing the contents of data words for a specific lane, in a readout frame
//!
//! After a frame ends, this data can then be parsed into ALPIDE words associated with a specific lane.

use super::{
    data_words::{ib_data_word_id_to_lane, ob_data_word_id_to_lane},
    Layer,
};

/// Struct for storing the contents of data words for a specific lane, in a readout frame
#[derive(Default)]
pub struct LaneDataFrame {
    /// The lane ID
    lane_id: u8,
    /// The data contents of data words (the 9 bytes of data excluding the ID)
    lane_data: Vec<u8>,
}

impl LaneDataFrame {
    /// Create a [LaneDataFrame] instance from a lane ID and Alpide data of the data frame.
    pub fn new(lane_id: u8, lane_data: Vec<u8>) -> Self {
        Self { lane_id, lane_data }
    }

    /// ID of the lane corresponding to the [Lane Data Frame][LaneDataFrame].
    pub fn id(&self) -> u8 {
        self.lane_id
    }

    /// Get a borrowed slice of the Alpide data of the [Lane Data Frame][LaneDataFrame].
    pub fn data(&self) -> &[u8] {
        &self.lane_data
    }

    /// Add data to the lane data. Useful for incrementally building the [LaneDataFrame] as data is processed.
    pub fn append_data(&mut self, data: &[u8]) {
        self.lane_data.extend_from_slice(data);
    }

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
