//! Validator for [Tdt]
use super::StatusWordValidator;
use crate::words::its::status_words::{tdt::Tdt, StatusWord};
use std::fmt::Write;

/// Validator for [Tdt]
#[derive(Debug, Copy, Clone)]
pub(super) struct TdtValidator;

impl StatusWordValidator<Tdt> for TdtValidator {
    fn sanity_check(tdt: &Tdt) -> Result<(), String> {
        let mut err_str = String::new();
        if tdt.id() != Tdt::ID {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tdt_validator() {
        let raw_data_tdt = [
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x01,
            Tdt::ID,
        ];
        let tdt = Tdt::load(&mut raw_data_tdt.as_slice()).unwrap();
        assert!(TdtValidator::sanity_check(&tdt).is_ok());
        let raw_data_tdt_bad_reserved = [
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0xF1,
            Tdt::ID,
        ];
        let tdt_bad = Tdt::load(&mut raw_data_tdt_bad_reserved.as_slice()).unwrap();

        assert!(TdtValidator::sanity_check(&tdt_bad).is_err());
    }

    #[test]
    #[should_panic]
    fn test_invalidate_tdt() {
        let raw_data_tdt_bad_id = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01];
        let tdt = Tdt::load(&mut raw_data_tdt_bad_id.as_slice()).unwrap();
        TdtValidator::sanity_check(&tdt).unwrap();
    }

    #[test]
    fn tdt_error_msg() {
        let raw_data_tdt_bad_id = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01];
        let tdt = Tdt::load(&mut raw_data_tdt_bad_id.as_slice()).unwrap();
        println!("{tdt:#?}");
        let err = TdtValidator::sanity_check(&tdt).err();
        println!("{err:?}");
        assert!(err.unwrap().contains("ID is not 0xF0: 0x"));
    }
}
