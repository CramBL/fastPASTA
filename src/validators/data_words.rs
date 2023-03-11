use crate::words::data_words::*;
use std::fmt::Write;

pub const DATA_WORD_SANITY_CHECKER: DataWordSanityChecker = DataWordSanityChecker {};
pub struct DataWordSanityChecker {}

impl DataWordSanityChecker {
    #[inline]
    pub fn check_any(&self, data_word: &[u8]) -> Result<(), String> {
        let mut err_str = String::new();
        let id = data_word[9];

        if !self.is_valid_any_id(id) {
            write!(err_str, "ID is invalid: {id:#02X} Full Word: ").unwrap();
            data_word.iter().for_each(|b| {
                write!(err_str, "{b:02X} ").unwrap();
            });
            err_str.push_str("[79:0]");
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
