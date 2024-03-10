//! Validator for [Ihw]
use std::fmt::Write;

use super::StatusWordValidator;
use crate::words::its::status_words::{ihw::Ihw, StatusWord};

/// Validator for [Ihw]
#[derive(Debug, Copy, Clone)]
pub struct IhwValidator;

impl StatusWordValidator<Ihw> for IhwValidator {
    fn sanity_check(ihw: &Ihw) -> Result<(), String> {
        let mut err_str = String::new();

        if ihw.id() != Ihw::ID {
            write!(err_str, "ID is not 0xE0: {:#2X} ", ihw.id()).unwrap();
            // Early return if ID is wrong
            return Err(err_str);
        }

        if !ihw.is_reserved_0() {
            write!(err_str, "reserved bits are not 0: {:2X} ", ihw.reserved()).unwrap();
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
    fn test_ihw_validator() {
        let raw_data_ihw = [0xFF, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0];
        let ihw = Ihw::load(&mut raw_data_ihw.as_slice()).unwrap();
        assert!(IhwValidator::sanity_check(&ihw).is_ok());
        let raw_data_ihw_bad_reserved =
            [0xFF, 0x3F, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0xE0];
        let ihw_bad = Ihw::load(&mut raw_data_ihw_bad_reserved.as_slice()).unwrap();
        assert!(IhwValidator::sanity_check(&ihw_bad).is_err());
    }

    #[test]
    fn test_invalidate_ihw() {
        let raw_data_ihw_bad_id = [0xFF, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xB0];
        let ihw = Ihw::load(&mut raw_data_ihw_bad_id.as_slice()).unwrap();
        let err = IhwValidator::sanity_check(&ihw).err();
        assert!(err.is_some());
        assert!(err.unwrap().contains("ID is not 0xE0: 0x"));
    }
}
