//! Validator for [Tdt]
use std::fmt::Write;

use super::StatusWordValidator;
use crate::words::its::status_words::{StatusWord, Tdt};

/// Validator for [Tdt]
#[derive(Debug, Copy, Clone)]
pub(super) struct TdtValidator {
    valid_id: u8,
}
impl StatusWordValidator<Tdt> for TdtValidator {
    fn sanity_check(&self, tdt: &Tdt) -> Result<(), String> {
        let mut err_str = String::new();
        if tdt.id() != self.valid_id {
            write!(err_str, "ID is not 0xF0: {:#2X} ", tdt.id()).unwrap();
            // Early return if ID is wrong
            return Err(err_str);
        }

        if !tdt.is_reserved_0() {
            write!(err_str, "reserved bits are not 0").unwrap();
        }

        if err_str.is_empty() {
            Ok(())
        } else {
            Err(err_str)
        }
    }
}

impl TdtValidator {
    pub const fn new_const() -> Self {
        Self { valid_id: 0xF0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TDT_VALIDATOR: TdtValidator = TdtValidator::new_const();

    #[test]
    fn test_tdt_validator() {
        let raw_data_tdt = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF0];
        let tdt = Tdt::load(&mut raw_data_tdt.as_slice()).unwrap();
        assert!(TDT_VALIDATOR.sanity_check(&tdt).is_ok());
        let raw_data_tdt_bad_reserved =
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF1, 0xF0];
        let tdt_bad = Tdt::load(&mut raw_data_tdt_bad_reserved.as_slice()).unwrap();

        assert!(TDT_VALIDATOR.sanity_check(&tdt_bad).is_err());
    }

    #[test]
    #[should_panic]
    fn test_invalidate_tdt() {
        let raw_data_tdt_bad_id = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01];
        let tdt = Tdt::load(&mut raw_data_tdt_bad_id.as_slice()).unwrap();
        TDT_VALIDATOR.sanity_check(&tdt).unwrap();
    }

    #[test]
    fn tdt_error_msg() {
        let raw_data_tdt_bad_id = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01];
        let tdt = Tdt::load(&mut raw_data_tdt_bad_id.as_slice()).unwrap();
        println!("{tdt:#?}");
        let err = TDT_VALIDATOR.sanity_check(&tdt).err();
        println!("{err:?}");
        assert!(err.unwrap().contains("ID is not 0xF0: 0x"));
    }
}
