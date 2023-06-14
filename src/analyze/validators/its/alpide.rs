pub mod alpide_lane_frame_decoder;
pub mod alpide_readout_frame;

use super::alpide::{
    alpide_lane_frame_decoder::AlpideLaneFrameDecoder, alpide_readout_frame::AlpideReadoutFrame,
};

/// Process ALPIDE data for a readout frame, per lane.
pub fn check_alpide_data_frame(mut alpide_readout_frame: AlpideReadoutFrame) -> Vec<(u8, String)> {
    let mut lane_error_msgs: Vec<(u8, String)> = Vec::new();
    let from_layer = alpide_readout_frame.is_from_layer();
    alpide_readout_frame
        .lane_data_frames
        .drain(..)
        .for_each(|lane_data_frame| {
            // Process data for each lane
            // New decoder for each lane
            let mut decoder = AlpideLaneFrameDecoder::new(from_layer);
            let lane_number = lane_data_frame.lane_number(from_layer);
            log::trace!("Processing lane #{lane_number}");

            if let Err(error_msgs) = decoder.analyze_alpide_frame(lane_data_frame) {
                let mut lane_error_string = format!("\n\tLane {lane_number} errors: ");
                error_msgs.for_each(|err| {
                    lane_error_string.push_str(&err);
                });
                lane_error_msgs.push((lane_number, lane_error_string));
            };
        });
    lane_error_msgs
}
