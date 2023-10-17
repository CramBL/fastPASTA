//! Performs sanity checks on data words
use crate::words::its::data_words as dw;
use std::fmt::Write;

pub mod ob;

/// Convenience const struct to avoid having to instantiate the struct elsewhere
pub const DATA_WORD_SANITY_CHECKER: DataWordSanityChecker = DataWordSanityChecker {};
/// Performs sanity checks on data words
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataWordSanityChecker;

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
        dw::VALID_IL_ID.contains(&id)
    }
    #[inline]
    fn is_valid_ml_id(&self, id: u8) -> bool {
        dw::VALID_ML_CONNECT0_ID.contains(&id)
            || dw::VALID_ML_CONNECT1_ID.contains(&id)
            || dw::VALID_ML_CONNECT2_ID.contains(&id)
            || dw::VALID_ML_CONNECT3_ID.contains(&id)
    }
    #[inline]
    fn is_valid_ol_id(&self, id: u8) -> bool {
        dw::VALID_OL_CONNECT0_ID.contains(&id)
            || dw::VALID_OL_CONNECT1_ID.contains(&id)
            || dw::VALID_OL_CONNECT2_ID.contains(&id)
            || dw::VALID_OL_CONNECT3_ID.contains(&id)
    }
    #[inline]
    fn is_valid_any_id(&self, id: u8) -> bool {
        self.is_valid_il_id(id) || self.is_valid_ml_id(id) || self.is_valid_ol_id(id)
    }
}
