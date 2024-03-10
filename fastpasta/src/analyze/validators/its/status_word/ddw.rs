//! Validator for [Ddw0]
use std::fmt::Write;

use super::StatusWordValidator;
use crate::words::its::status_words::{ddw::Ddw0, StatusWord};

/// Validator for [Ddw0]
#[derive(Debug, Copy, Clone)]
pub struct Ddw0Validator;

impl StatusWordValidator<Ddw0> for Ddw0Validator {
    fn sanity_check(ddw0: &Ddw0) -> Result<(), String> {
        let mut err_str = String::new();

        if ddw0.id() != Ddw0::ID {
            write!(err_str, "ID is not 0xE4: {:#2X} ", ddw0.id()).unwrap();
            // Early return if ID is wrong
            return Err(err_str);
        }

        if !ddw0.is_reserved_0() {
            write!(
                err_str,
                "reserved bits are not 0:  {:b} {:b} ",
                ddw0.reserved0_1(),
                ddw0.reserved2(),
            )
            .unwrap();
        }

        if ddw0.index() != 0 {
            write!(err_str, "index is not 0:  {:#2X} ", ddw0.index()).unwrap();
        }

        if !err_str.is_empty() {
            return Err(err_str);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ddw0_valid() {
        let raw_data_ddw0 = [
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            Ddw0::ID,
        ];

        let ddw0 = Ddw0::load(&mut raw_data_ddw0.as_slice()).unwrap();
        assert!(Ddw0Validator::sanity_check(&ddw0).is_ok());

        // Atypical TDT, some lane errors and warnings etc.
        const LANE_0_AND_3_IN_WARNING: u8 = 0b0100_0001;
        const LANE_4_TO_7_IN_FATAL: u8 = 0b1111_1111;
        const LANE_8_TO_11_IN_WARNING: u8 = 0b0101_0101;
        const LANE_12_AND_15_IN_ERROR: u8 = 0b1000_0010;
        const LANE_16_AND_19_IN_OK: u8 = 0b0000_0000;
        const LANE_22_IN_WARNING: u8 = 0b0001_0000;
        const LANE_24_AND_25_IN_ERROR: u8 = 0b0000_1010;
        const RESERVED0: u8 = 0x00;
        const TRANSMISSION_TO_LANE_STARTS_VIOLATION_SET: u8 = 0x0A;

        let raw_data_ddw0_new = [
            LANE_0_AND_3_IN_WARNING,
            LANE_4_TO_7_IN_FATAL,
            LANE_8_TO_11_IN_WARNING,
            LANE_12_AND_15_IN_ERROR,
            LANE_16_AND_19_IN_OK,
            LANE_22_IN_WARNING,
            LANE_24_AND_25_IN_ERROR,
            RESERVED0,
            TRANSMISSION_TO_LANE_STARTS_VIOLATION_SET,
            Ddw0::ID,
        ];

        let ddw0_new = Ddw0::load(&mut raw_data_ddw0_new.as_slice()).unwrap();
        println!("{ddw0:#?}");
        assert!(Ddw0Validator::sanity_check(&ddw0_new).is_ok());
    }
}
