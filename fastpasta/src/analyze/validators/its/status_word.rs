//! Validators for status words: [IHW][Ihw], [TDH][Tdh], [TDT][Tdt] & [DDW0][Ddw0].
//!
//! Each validator is aggregated by the [StatusWordSanityChecker] struct.

use crate::util::*;
use ddw::Ddw0Validator;
use ihw::IhwValidator;
use tdh::TdhValidator;
use tdt::TdtValidator;

mod ddw;
mod ihw;
pub(super) mod tdh;
pub(super) mod tdt;
pub mod util;

/// Encapsulates all status word validators.
#[derive(Debug, Clone, Copy)]
pub struct StatusWordSanityChecker;

impl StatusWordSanityChecker {
    /// Checks if argument is a valid [IHW][Ihw] status word.
    pub fn check_ihw(ihw: &Ihw) -> Result<(), String> {
        IhwValidator::sanity_check(ihw)
    }
    /// Checks if argument is a valid [TDH][Tdh] status word.
    pub fn check_tdh(tdh: &Tdh) -> Result<(), String> {
        TdhValidator::sanity_check(tdh)
    }
    /// Checks if argument is a valid [TDT][Tdt] status word.
    pub fn check_tdt(tdt: &Tdt) -> Result<(), String> {
        TdtValidator::sanity_check(tdt)
    }
    /// Checks if argument is a valid [DDW0][Ddw0] status word.
    pub fn check_ddw0(ddw0: &Ddw0) -> Result<(), String> {
        Ddw0Validator::sanity_check(ddw0)
    }
}

/// Abstraction of a status word validator, each validator should implement this interface.
pub trait StatusWordValidator<T: StatusWord> {
    /// Perform a sanity check on a [Status Word][StatusWord].
    ///
    /// # Returns
    /// [Ok] if the sanity check passes
    ///
    /// # Errors
    /// If a check fails, returns [Err(String)][Err] where the string describes the sanity check that failed
    fn sanity_check(status_word: &T) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_ddw0_invalid() {
        let raw_ddw0_bad_index = [
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x10,
            Ddw0::ID,
        ];

        let ddw0_bad = Ddw0::load(&mut raw_ddw0_bad_index.as_slice()).unwrap();
        StatusWordSanityChecker::check_ddw0(&ddw0_bad).unwrap();
    }

    #[test]
    fn test_ddw0_invalid_handled() {
        let raw_ddw0_bad_index = [
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x10,
            Ddw0::ID,
        ];

        let ddw0_bad = Ddw0::load(&mut raw_ddw0_bad_index.as_slice()).unwrap();
        assert!(StatusWordSanityChecker::check_ddw0(&ddw0_bad).is_err());
    }

    #[test]
    fn test_ddw0_error_msg_bad_index() {
        let raw_ddw0_bad_index = [
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x10,
            Ddw0::ID,
        ];

        let ddw0_bad = Ddw0::load(&mut raw_ddw0_bad_index.as_slice()).unwrap();
        let err = StatusWordSanityChecker::check_ddw0(&ddw0_bad).err();
        eprintln!("{:?}", err);
        assert!(err.unwrap().contains("index is not 0"));
    }
    #[test]
    fn test_ddw0_error_msg_bad_id() {
        let raw_data_ddw0_bad_id = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14];

        let ddw0_bad = Ddw0::load(&mut raw_data_ddw0_bad_id.as_slice()).unwrap();
        let err = StatusWordSanityChecker::check_ddw0(&ddw0_bad).err();
        eprintln!("{:?}", err);
        assert!(err.unwrap().contains("ID is not 0xE4: 0x"));
    }
}
