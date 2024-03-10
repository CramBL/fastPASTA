//! Contains the validator for outer barrel data words

use crate::words::its::{
    data_words::{ob_data_word_id_to_input_number_connector, ob_data_word_id_to_lane},
    status_words::util::is_lane_active,
};

/// Performs checks on a Data word from Outer Barrel
#[derive(Debug, Default, Clone, Copy)]
pub struct ObDataWordValidator;

impl ObDataWordValidator {
    /// Performs checks on a data word from outer barrell.
    ///
    /// If any checks fail, an [Err] is returned containing a vector of error messages
    pub fn check(ob_data_word_slice: &[u8], ihw_active_lanes: u32) -> Result<(), Vec<String>> {
        let mut errors = Vec::<String>::new();

        let lane_id = ob_data_word_id_to_lane(ob_data_word_slice[9]);

        // Check the lane is active according to the IHW
        if !is_lane_active(lane_id, ihw_active_lanes) {
            errors.push(format!("[E71] OB lane {lane_id} is not active according to IHW active_lanes: {ihw_active_lanes:#X}."));
        }

        // Check that the lane input connector value is valid
        let input_number_connector =
            ob_data_word_id_to_input_number_connector(ob_data_word_slice[9]);

        if input_number_connector > 6 {
            errors.push(format!(
                "[E73] OB Data Word has input connector {input_number_connector} > 6."
            ))
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
