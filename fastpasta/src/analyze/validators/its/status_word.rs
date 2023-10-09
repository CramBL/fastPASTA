//! Validators for status words: [IHW][Ihw], [TDH][Tdh], [TDT][Tdt] & [DDW0][Ddw0].
//!
//! Each validator is aggregated by the [StatusWordSanityChecker] struct.

use crate::words::its::status_words::{Ddw0, Ihw, StatusWord, Tdh, Tdt};

use ddw::Ddw0Validator;
use ihw::IhwValidator;
use tdh::TdhValidator;
use tdt::TdtValidator;

mod ddw;
mod ihw;
mod tdh;
mod tdt;

/// Convenience const for the [StatusWordSanityChecker].
pub const STATUS_WORD_SANITY_CHECKER: StatusWordSanityChecker = StatusWordSanityChecker::new();

/// Aggregates all status word validators.
#[derive(Debug, Clone, Copy)]
pub struct StatusWordSanityChecker {
    ihw_validator: IhwValidator,
    tdh_validator: TdhValidator,
    tdt_validator: TdtValidator,
    ddw0_validator: Ddw0Validator,
    // No checks for CDW
}
impl StatusWordSanityChecker {
    /// Creates a new [StatusWordSanityChecker] in a const context.
    pub const fn new() -> Self {
        Self {
            ihw_validator: IhwValidator::new_const(),
            tdh_validator: TdhValidator::new_const(),
            tdt_validator: TdtValidator::new_const(),
            ddw0_validator: Ddw0Validator::new_const(),
        }
    }

    /// Checks if argument is a valid [IHW][Ihw] status word.
    pub fn sanity_check_ihw(&self, ihw: &Ihw) -> Result<(), String> {
        self.ihw_validator.sanity_check(ihw)
    }
    /// Checks if argument is a valid [TDH][Tdh] status word.
    pub fn sanity_check_tdh(&self, tdh: &Tdh) -> Result<(), String> {
        self.tdh_validator.sanity_check(tdh)
    }
    /// Checks if argument is a valid [TDT][Tdt] status word.
    pub fn sanity_check_tdt(&self, tdt: &Tdt) -> Result<(), String> {
        self.tdt_validator.sanity_check(tdt)
    }
    /// Checks if argument is a valid [DDW0][Ddw0] status word.
    pub fn sanity_check_ddw0(&self, ddw0: &Ddw0) -> Result<(), String> {
        self.ddw0_validator.sanity_check(ddw0)
    }
}

trait StatusWordValidator<T: StatusWord> {
    fn sanity_check(&self, status_word: &T) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::words::its::status_words::StatusWord;

    #[test]
    #[should_panic]
    fn test_ddw0_invalid() {
        let raw_ddw0_bad_index = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0xE4];

        let ddw0_bad = Ddw0::load(&mut raw_ddw0_bad_index.as_slice()).unwrap();
        STATUS_WORD_SANITY_CHECKER
            .sanity_check_ddw0(&ddw0_bad)
            .unwrap();
    }

    #[test]
    fn test_ddw0_invalid_handled() {
        let raw_ddw0_bad_index = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0xE4];

        let ddw0_bad = Ddw0::load(&mut raw_ddw0_bad_index.as_slice()).unwrap();
        assert!(STATUS_WORD_SANITY_CHECKER
            .sanity_check_ddw0(&ddw0_bad)
            .is_err());
    }

    #[test]
    fn test_ddw0_error_msg_bad_index() {
        let raw_ddw0_bad_index = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0xE4];

        let ddw0_bad = Ddw0::load(&mut raw_ddw0_bad_index.as_slice()).unwrap();
        let err = STATUS_WORD_SANITY_CHECKER
            .sanity_check_ddw0(&ddw0_bad)
            .err();
        eprintln!("{:?}", err);
        assert!(err.unwrap().contains("index is not 0"));
    }
    #[test]
    fn test_ddw0_error_msg_bad_id() {
        let raw_data_ddw0_bad_id = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14];

        let ddw0_bad = Ddw0::load(&mut raw_data_ddw0_bad_id.as_slice()).unwrap();
        let err = STATUS_WORD_SANITY_CHECKER
            .sanity_check_ddw0(&ddw0_bad)
            .err();
        eprintln!("{:?}", err);
        assert!(err.unwrap().contains("ID is not 0xE4: 0x"));
    }
}
