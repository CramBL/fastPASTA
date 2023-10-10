//! Validator for [Ihw]
use std::fmt::Write;

use super::StatusWordValidator;
use crate::words::its::status_words::{Ihw, StatusWord};

/// Validator for [Ihw]
#[derive(Debug, Copy, Clone)]
pub struct IhwValidator {
    valid_id: u8,
}

impl StatusWordValidator<Ihw> for IhwValidator {
    fn sanity_check(&self, ihw: &Ihw) -> Result<(), String> {
        let mut err_str = String::new();

        if ihw.id() != self.valid_id {
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

impl Default for IhwValidator {
    fn default() -> Self {
        Self { valid_id: 0xE0 }
    }
}

impl IhwValidator {
    /// Constant initialize a new [IhwValidator]
    pub const fn new_const() -> Self {
        Self { valid_id: 0xE0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IHW_VALIDATOR: IhwValidator = IhwValidator::new_const();

    #[test]
    fn test_ihw_validator() {
        let raw_data_ihw = [0xFF, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0];
        let ihw = Ihw::load(&mut raw_data_ihw.as_slice()).unwrap();
        assert!(IHW_VALIDATOR.sanity_check(&ihw).is_ok());
        let raw_data_ihw_bad_reserved =
            [0xFF, 0x3F, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0xE0];
        let ihw_bad = Ihw::load(&mut raw_data_ihw_bad_reserved.as_slice()).unwrap();
        assert!(IHW_VALIDATOR.sanity_check(&ihw_bad).is_err());
    }

    #[test]
    fn test_invalidate_ihw() {
        let raw_data_ihw_bad_id = [0xFF, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xB0];
        let ihw = Ihw::load(&mut raw_data_ihw_bad_id.as_slice()).unwrap();
        let err = IHW_VALIDATOR.sanity_check(&ihw).err();
        assert!(err.is_some());
        assert!(err.unwrap().contains("ID is not 0xE0: 0x"));
    }
}
