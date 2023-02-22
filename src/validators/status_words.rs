use crate::words::status_words::{Cdw, Ddw0, Ihw, StatusWord, Tdh, Tdt};
use std::fmt::Write;

/// Validators for status words: `IHW`, `TDH`, `TDT`, `DDW0`
///
/// The `StatusWordValidator` trait is implemented for each status word type.
///
/// The `sanity_check` method is used to check the status word for errors.
///
/// Each validator is aggregated by the `StatusWordSanityChecker` struct.

const STATUS_WORD_SANITY_CHECKER: StatusWordSanityChecker = StatusWordSanityChecker::new();
pub struct StatusWordSanityChecker {
    ihw_validator: IhwValidator,
    tdh_validator: TdhValidator,
    tdt_validator: TdtValidator,
    ddw0_validator: Ddw0Validator,
}
impl StatusWordSanityChecker {
    pub const fn new() -> Self {
        Self {
            ihw_validator: IHW_VALIDATOR,
            tdh_validator: TDH_VALIDATOR,
            tdt_validator: TDT_VALIDATOR,
            ddw0_validator: DDW0_VALIDATOR,
        }
    }

    pub fn sanity_check_ihw(&self, ihw: &Ihw) -> Result<(), String> {
        self.ihw_validator.sanity_check(ihw)
    }
    pub fn sanity_check_tdh(&self, tdh: &Tdh) -> Result<(), String> {
        self.tdh_validator.sanity_check(tdh)
    }
    pub fn sanity_check_tdt(&self, tdt: &Tdt) -> Result<(), String> {
        self.tdt_validator.sanity_check(tdt)
    }
    pub fn sanity_check_ddw0(&self, ddw0: &Ddw0) -> Result<(), String> {
        self.ddw0_validator.sanity_check(ddw0)
    }
}

pub trait StatusWordValidator<T: StatusWord> {
    fn sanity_check(&self, status_word: &T) -> Result<(), String>;
}

const IHW_VALIDATOR: IhwValidator = IhwValidator {};
pub struct IhwValidator {}
impl StatusWordValidator<Ihw> for IhwValidator {
    fn sanity_check(&self, ihw: &Ihw) -> Result<(), String> {
        let mut err_str = String::new();
        let mut err_cnt: u8 = 0;
        if ihw.is_reserved_0() == false {
            err_cnt += 1;
            write!(err_str, "reserved bits are not 0:  {:b}", ihw.reserved()).unwrap();
        }
        if err_cnt > 0 {
            Err(err_str)
        } else {
            Ok(())
        }
    }
}

const TDH_VALIDATOR: TdhValidator = TdhValidator {};
pub struct TdhValidator {}
impl StatusWordValidator<Tdh> for TdhValidator {
    fn sanity_check(&self, tdh: &Tdh) -> Result<(), String> {
        let mut err_str = String::new();
        let mut err_cnt: u8 = 0;
        if tdh.is_reserved_0() == false {
            err_cnt += 1;
            write!(
                err_str,
                "reserved bits are not 0:  {:b} {:b} {:b}",
                tdh.reserved0(),
                tdh.reserved1(),
                tdh.reserved2()
            )
            .unwrap();
        }

        // Trigger Orbit ID check
        //if tdh.trigger_orbit == 0

        // Trigger Bunch Crossing ID check

        // Trigger Type check (12 lowest bits of trigger type received from CTP)
        // All values are valid except 0x0
        if tdh.trigger_type() == 0 {
            err_cnt += 1;
            write!(err_str, "trigger type is 0").unwrap();
        }

        if err_cnt > 0 {
            Err(err_str)
        } else {
            Ok(())
        }
    }
}

const TDT_VALIDATOR: TdtValidator = TdtValidator {};
pub struct TdtValidator {}
impl StatusWordValidator<Tdt> for TdtValidator {
    fn sanity_check(&self, tdt: &Tdt) -> Result<(), String> {
        let mut err_str = String::new();
        let mut err_cnt: u8 = 0;
        if tdt.is_reserved_0() == false {
            err_cnt += 1;
            write!(
                err_str,
                "reserved bits are not 0:  {:b} {:b} {:b}",
                tdt.reserved0(),
                tdt.reserved1(),
                tdt.reserved2()
            )
            .unwrap();
        }

        if err_cnt > 0 {
            Err(err_str)
        } else {
            Ok(())
        }
    }
}

const DDW0_VALIDATOR: Ddw0Validator = Ddw0Validator {}; // Used in the final StatusWord sanity checker
pub struct Ddw0Validator {}
impl StatusWordValidator<Ddw0> for Ddw0Validator {
    fn sanity_check(&self, ddw0: &Ddw0) -> Result<(), String> {
        let mut err_str = String::new();
        let mut err_cnt: u8 = 0;
        if ddw0.is_reserved_0() == false {
            err_cnt += 1;
            write!(
                err_str,
                "reserved bits are not 0:  {:b} {:b}",
                ddw0.reserved0_1(),
                ddw0.reserved2(),
            )
            .unwrap();
        }

        if ddw0.index() != 0 {
            err_cnt += 1;
            write!(err_str, "index is not 0:  {:b}", ddw0.index()).unwrap();
        }

        if err_cnt > 0 {
            Err(err_str)
        } else {
            Ok(())
        }
    }
}
// No checks for CDW

#[cfg(test)]
mod tests {
    use super::*;
    use crate::words::status_words::{Ihw, StatusWord, Tdh, Tdt};

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
    fn test_tdh_validator() {
        let raw_data_tdh = [0x03, 0x1A, 0x00, 0x00, 0x75, 0xD5, 0x7D, 0x0B, 0x00, 0xE8];
        let tdh = Tdh::load(&mut raw_data_tdh.as_slice()).unwrap();
        assert!(TDH_VALIDATOR.sanity_check(&tdh).is_ok());
        let raw_data_tdh_bad_reserved =
            [0x03, 0x1A, 0x00, 0x00, 0x75, 0xD5, 0x7D, 0x0B, 0x0F, 0xE8];
        let tdh_bad = Tdh::load(&mut raw_data_tdh_bad_reserved.as_slice()).unwrap();
        assert!(TDH_VALIDATOR.sanity_check(&tdh_bad).is_err());
    }

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
    fn test_ddw0_valid() {
        let raw_data_ddw0 = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE4];

        let ddw0 = Ddw0::load(&mut raw_data_ddw0.as_slice()).unwrap();
        assert!(DDW0_VALIDATOR.sanity_check(&ddw0).is_ok());

        const VALID_ID: u8 = 0xE4;
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

        let raw_data_ddw0 = [
            LANE_0_AND_3_IN_WARNING,
            LANE_4_TO_7_IN_FATAL,
            LANE_8_TO_11_IN_WARNING,
            LANE_12_AND_15_IN_ERROR,
            LANE_16_AND_19_IN_OK,
            LANE_22_IN_WARNING,
            LANE_24_AND_25_IN_ERROR,
            RESERVED0,
            TRANSMISSION_TO_LANE_STARTS_VIOLATION_SET,
            0xE4,
        ];

        let ddw0 = Ddw0::load(&mut raw_data_ddw0.as_slice()).unwrap();
        assert!(DDW0_VALIDATOR.sanity_check(&ddw0).is_ok());
    }

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
        match STATUS_WORD_SANITY_CHECKER.sanity_check_ddw0(&ddw0_bad) {
            Ok(_) => assert!(false),
            Err(e) => println!("DDW0 sanity check failed with: {}", e),
        }
    }
}
