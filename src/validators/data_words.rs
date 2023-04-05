//! Performs sanity checks on data words
use crate::words::data_words::*;
use std::fmt::Write;

/// Convenience const struct to avoid having to instantiate the struct elsewhere
pub const DATA_WORD_SANITY_CHECKER: DataWordSanityChecker = DataWordSanityChecker {};
/// Performs sanity checks on data words
pub struct DataWordSanityChecker {}

impl DataWordSanityChecker {
    /// Checks is a valid IL/ML/OL data word
    #[inline]
    pub fn check_any(&self, data_word: &[u8]) -> Result<(), String> {
        let mut err_str = String::new();
        let id = data_word[9];

        if !self.is_valid_any_id(id) {
            write!(err_str, "ID is invalid: {id:#02X}").unwrap();
            // Early return if ID is wrong
            return Err(err_str);
        }
        Ok(())
    }

    #[inline]
    fn is_valid_il_id(&self, id: u8) -> bool {
        id >= VALID_IL_ID_MIN_MAX.0 && id <= VALID_IL_ID_MIN_MAX.1
    }
    #[inline]
    fn is_valid_ml_id(&self, id: u8) -> bool {
        id >= VALID_ML_CONNECT0_ID_MIN_MAX.0 && id <= VALID_ML_CONNECT0_ID_MIN_MAX.1
            || id >= VALID_ML_CONNECT1_ID_MIN_MAX.0 && id <= VALID_ML_CONNECT1_ID_MIN_MAX.1
            || id >= VALID_ML_CONNECT2_ID_MIN_MAX.0 && id <= VALID_ML_CONNECT2_ID_MIN_MAX.1
            || id >= VALID_ML_CONNECT3_ID_MIN_MAX.0 && id <= VALID_ML_CONNECT3_ID_MIN_MAX.1
    }
    #[inline]
    fn is_valid_ol_id(&self, id: u8) -> bool {
        id >= VALID_OL_CONNECT0_ID_MIN_MAX.0 && id <= VALID_OL_CONNECT0_ID_MIN_MAX.1
            || id >= VALID_OL_CONNECT1_ID_MIN_MAX.0 && id <= VALID_OL_CONNECT1_ID_MIN_MAX.1
            || id >= VALID_OL_CONNECT2_ID_MIN_MAX.0 && id <= VALID_OL_CONNECT2_ID_MIN_MAX.1
            || id >= VALID_OL_CONNECT3_ID_MIN_MAX.0 && id <= VALID_OL_CONNECT3_ID_MIN_MAX.1
    }
    #[inline]
    fn is_valid_any_id(&self, id: u8) -> bool {
        self.is_valid_il_id(id) || self.is_valid_ml_id(id) || self.is_valid_ol_id(id)
    }
}
