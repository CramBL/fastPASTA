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

        let mut err_cnt: u8 = 0;
        if !ihw.is_reserved_0() {
            err_cnt += 1;
            write!(err_str, "reserved bits are not 0: {:2X} ", ihw.reserved()).unwrap();
        }
        if err_cnt > 0 {
            Err(err_str)
        } else {
            Ok(())
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
