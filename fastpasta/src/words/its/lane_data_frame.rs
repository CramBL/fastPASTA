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
