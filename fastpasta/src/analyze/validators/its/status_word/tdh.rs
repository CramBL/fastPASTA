//! Validator for [Ddw0]
use std::fmt::Write;

use super::StatusWordValidator;
use crate::words::its::status_words::{StatusWord, Tdh};

#[derive(Debug, Copy, Clone)]
pub struct TdhValidator {
    valid_id: u8,
}

impl StatusWordValidator<Tdh> for TdhValidator {
    fn sanity_check(&self, tdh: &Tdh) -> Result<(), String> {
        let mut err_str = String::new();

        if tdh.id() != self.valid_id {
            write!(err_str, "ID is not 0xE8: {:#2X} ", tdh.id()).unwrap();
            // Early return if ID is wrong
            return Err(err_str);
        }

        let mut err_cnt: u8 = 0;
        if !tdh.is_reserved_0() {
            err_cnt += 1;
            write!(
                err_str,
                "reserved bits are not 0:  {:X} {:X} {:X} ",
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
        if tdh.trigger_type() == 0 && tdh.internal_trigger() == 0 {
            err_cnt += 1;
            write!(err_str, "trigger type and internal trigger both 0").unwrap();
        }

        debug_assert!(tdh.internal_trigger() < 2);

        if err_cnt > 0 {
            Err(err_str)
        } else {
            Ok(())
        }
    }
}

impl TdhValidator {
    pub const fn new_const() -> Self {
        Self { valid_id: 0xE8 }
    }

    pub fn matches_trigger_interval(
        current_trg_bc: u16,
        previous_trg_bc: u16,
        specified_period: u16,
    ) -> Result<(), u16> {
        let detected_period = if current_trg_bc < previous_trg_bc {
            // Bunch Crossing ID wrapped around
            // +1 cause of incrementing the Orbit counter for the rollover
            let distance_to_max = Tdh::MAX_BC - previous_trg_bc + 1;
            distance_to_max + current_trg_bc
        } else {
            current_trg_bc - previous_trg_bc
        };
        if detected_period == specified_period {
            Ok(())
        } else {
            Err(detected_period)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TDH_VALIDATOR: TdhValidator = TdhValidator::new_const();

    #[test]
    fn test_tdh_validator() {
        let raw_data_tdh = [0x03, 0x1A, 0x00, 0x00, 0x75, 0xD5, 0x7D, 0x0B, 0x00, 0xE8];
        let tdh = Tdh::load(&mut raw_data_tdh.as_slice()).unwrap();
        println!("{tdh:#?}");
        assert!(TDH_VALIDATOR.sanity_check(&tdh).is_ok());
        let raw_data_tdh_bad_reserved =
            [0x03, 0x1A, 0x00, 0x00, 0x75, 0xD5, 0x7D, 0x0B, 0x0F, 0xE8];
        let tdh_bad = Tdh::load(&mut raw_data_tdh_bad_reserved.as_slice()).unwrap();
        assert!(TDH_VALIDATOR.sanity_check(&tdh_bad).is_err());
    }

    #[test]
    #[should_panic]
    fn test_tdh_validator_bad_id() {
        let raw_data_tdh_bad_id = [0x03, 0x1A, 0x00, 0x00, 0x75, 0xD5, 0x7D, 0x0B, 0x00, 0xE7];
        let tdh = Tdh::load(&mut raw_data_tdh_bad_id.as_slice()).unwrap();
        TDH_VALIDATOR.sanity_check(&tdh).unwrap();
    }

    #[test]
    fn test_tdh_err_msg() {
        let raw_data_tdh_bad_id = [0x03, 0x1A, 0x00, 0x00, 0x75, 0xD5, 0x7D, 0x0B, 0x00, 0xE7];
        let tdh = Tdh::load(&mut raw_data_tdh_bad_id.as_slice()).unwrap();
        let err = TDH_VALIDATOR.sanity_check(&tdh).err();
        println!("{err:?}");
        assert!(err.unwrap().contains("ID is not 0xE8: 0x"));
    }
}
