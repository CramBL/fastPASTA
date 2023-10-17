//! Contains the validator for outer barrel data words

use crate::words::its::status_words::util::is_lane_active;

/// Performs checks on a Data word from Inner Barrel
#[derive(Debug, Default, Clone, Copy)]
pub struct IbDataWordValidator;

impl IbDataWordValidator {
    /// Constant initialize a [IbDataWordValidator]
    pub const fn new_const() -> Self {
        Self {}
    }

    /// Checks validity of an Inner Barrel data word
    ///
    /// If the check fails, returns an [Err] containing the error message
    // Note: Change return to a vector of String if more checks are added
    pub fn check(ib_data_word: &[u8], ihw_active_lanes: u32) -> Result<(), Box<str>> {
        let lane_id = ib_data_word[9] & 0x1F;
        // lane in active_lanes;
        if is_lane_active(lane_id, ihw_active_lanes) {
            Ok(())
        } else {
            Err(format!("[E72] IB lane {lane_id} is not active according to IHW active_lanes: {ihw_active_lanes:#X}.").into())
        }
    }
}
