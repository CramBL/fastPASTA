//! Validator for [Ddw0]
use std::fmt::Write;

use alice_protocol_reader::rdh::RDH;

use super::{util::StatusWordContainer, StatusWordValidator};
use crate::words::its::status_words::{tdh::Tdh, StatusWord};

#[derive(Debug, Copy, Clone)]
pub struct TdhValidator;

impl StatusWordValidator<Tdh> for TdhValidator {
    fn sanity_check(tdh: &Tdh) -> Result<(), String> {
        let mut err_str = String::new();

        if tdh.id() != Tdh::ID {
            write!(err_str, "ID is not 0xE8: {:#2X} ", tdh.id()).unwrap();
            // Early return if ID is wrong
            return Err(err_str);
        }

        if !tdh.is_reserved_0() {
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
            write!(err_str, "trigger type and internal trigger both 0").unwrap();
        }

        debug_assert!(tdh.internal_trigger() < 2);

        if err_str.is_empty() {
            Ok(())
        } else {
            Err(err_str)
        }
    }
}

impl TdhValidator {
    /// Checks if the TDH trigger_bc period matches the specified value
    ///
    /// reports an error with the detected erroneous period if the check fails
    ///
    /// The check is only applicable to consecutive TDHs with internal_trigger set.
    pub fn check_trigger_interval(
        tdh: &Tdh,
        prev_int_tdh: &Tdh,
        expect_period: u16,
    ) -> Result<(), String> {
        debug_assert!(tdh.internal_trigger() == 1 && prev_int_tdh.internal_trigger() == 1);
        if let Err(err_period) = Self::matches_trigger_interval(
            tdh.trigger_bc(),
            prev_int_tdh.trigger_bc(),
            expect_period,
        ) {
            Err(format!(
                "[E45] TDH trigger period mismatch with user specified: {expect_period} != {err_period}\
                                    \n\tPrevious TDH Orbit_BC: {prev_trigger_orbit}_{prev_trigger_bc:>4}\
                                    \n\tCurrent  TDH Orbit_BC: {current_trigger_orbit}_{current_trigger_bc:>4}",
                                    prev_trigger_orbit = prev_int_tdh.trigger_orbit(),
                                    prev_trigger_bc = prev_int_tdh.trigger_bc(),
                                    current_trigger_orbit = tdh.trigger_orbit(),
                                    current_trigger_bc = tdh.trigger_bc()
            ))
        } else {
            Ok(())
        }
    }

    /// Checks if the period between two TDH trigger_bc values matches a specified value
    ///
    /// returns an error with the detected erroneous period if the check fails
    ///
    /// The check is only applicable to consecutive TDHs with internal_trigger set.
    #[inline]
    fn matches_trigger_interval(
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

    /// Checks TDH trigger_bc following a TDT packet_done = 1
    ///
    /// Valid if current TDH trigger_bc >= previous TDH trigger_bc
    #[inline]
    pub fn check_after_tdt_packet_done_true(status_words: &StatusWordContainer) -> Result<(), ()> {
        if let Some(previous_tdh) = status_words.prv_tdh() {
            if previous_tdh.trigger_bc() > status_words.tdh().unwrap().trigger_bc() {
                return Err(());
            }
        }
        Ok(())
    }

    /// Checks TDH fields: continuation, orbit, when the TDH immediately follows an IHW.
    ///
    /// If any checks fail, returns `Err(Vec<ErrMsgs>)`
    #[inline]
    pub fn check_tdh_no_continuation(tdh: &Tdh, rdh: &impl RDH) -> Result<(), Vec<String>> {
        let mut errors = Vec::<String>::new();

        if tdh.continuation() != 0 {
            errors.push("[E42] TDH continuation is not 0".into());
        }

        if tdh.trigger_orbit() != rdh.rdh1().orbit {
            errors.push("[E444] TDH trigger_orbit is not equal to RDH orbit".into());
        }

        if rdh.pages_counter() == 0 && (tdh.internal_trigger() == 1 || rdh.rdh2().is_pht_trigger())
        {
            // check BC and trigger type match
            Self::check_tdh_rdh_bc_trigger_type_match(tdh, rdh, &mut errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// A TDH immediately following an IHW should have trigger and BC match the last seen RDH
    #[inline]
    fn check_tdh_rdh_bc_trigger_type_match(tdh: &Tdh, rdh: &impl RDH, errors: &mut Vec<String>) {
        // Check that the BC of the TDH and RDH match
        if tdh.trigger_bc() != rdh.rdh1().bc() {
            errors.push(format!("[E445] TDH trigger_bc is not equal to RDH bc, TDH: {tdh_trig_bc:#X}, RDH: {rdh_bc:#X}.",
                        tdh_trig_bc = tdh.trigger_bc(),
                    rdh_bc = rdh.rdh1().bc()));
        }

        // Now check that the trigger_type matches

        // TDH only has the 12 LSB of the trigger type
        let rdh_trigger_type_12_lsb = rdh.rdh2().trigger_type as u16 & 0xFFF;

        if rdh_trigger_type_12_lsb != tdh.trigger_type() {
            errors.push(format!(
                "[E44] TDH trigger_type {tdh_tt:#X} != {rdh_tt:#X} RDH trigger_type[11:0].",
                tdh_tt = tdh.trigger_type(),
                rdh_tt = rdh_trigger_type_12_lsb
            ));
        }
    }

    /// Checks TDH when expecting continuation (Previous TDT packet_done = 0).
    ///
    /// If there's a previous TDH, it is cross-checked with the current TDH.
    ///
    /// If any checks fail, returns [Err] containing a vector of error messages
    #[inline]
    pub fn check_continuation(tdh: &Tdh, prev_tdh: Option<&Tdh>) -> Result<(), Vec<String>> {
        let mut errors = Vec::<String>::new();

        if tdh.continuation() != 1 {
            errors.push("[E41] TDH continuation is not 1".into());
        }

        if let Some(prev_tdh) = prev_tdh {
            if tdh.trigger_bc() != prev_tdh.trigger_bc() {
                errors.push("[E441] TDH trigger_bc is not the same".into());
            }
            if tdh.trigger_orbit() != prev_tdh.trigger_orbit() {
                errors.push("[E442] TDH trigger_orbit is not the same".into());
            }
            if tdh.trigger_type() != prev_tdh.trigger_type() {
                errors.push("[E443] TDH trigger_type is not the same".into());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tdh_validator() {
        let raw_data_tdh = [
            0x03,
            0x1A,
            0x00,
            0x00,
            0x75,
            0xD5,
            0x7D,
            0x0B,
            0x00,
            Tdh::ID,
        ];
        let tdh = Tdh::load(&mut raw_data_tdh.as_slice()).unwrap();
        println!("{tdh:#?}");
        assert!(TdhValidator::sanity_check(&tdh).is_ok());
        let raw_data_tdh_bad_reserved = [
            0x03,
            0x1A,
            0x00,
            0x00,
            0x75,
            0xD5,
            0x7D,
            0x0B,
            0x0F,
            Tdh::ID,
        ];
        let tdh_bad = Tdh::load(&mut raw_data_tdh_bad_reserved.as_slice()).unwrap();
        assert!(TdhValidator::sanity_check(&tdh_bad).is_err());
    }

    #[test]
    #[should_panic]
    fn test_tdh_validator_bad_id() {
        let raw_data_tdh_bad_id = [0x03, 0x1A, 0x00, 0x00, 0x75, 0xD5, 0x7D, 0x0B, 0x00, 0xE7];
        let tdh = Tdh::load(&mut raw_data_tdh_bad_id.as_slice()).unwrap();
        TdhValidator::sanity_check(&tdh).unwrap();
    }

    #[test]
    fn test_tdh_err_msg() {
        let raw_data_tdh_bad_id = [0x03, 0x1A, 0x00, 0x00, 0x75, 0xD5, 0x7D, 0x0B, 0x00, 0xE7];
        let tdh = Tdh::load(&mut raw_data_tdh_bad_id.as_slice()).unwrap();
        let err = TdhValidator::sanity_check(&tdh).err();
        println!("{err:?}");
        assert!(err.unwrap().contains("ID is not 0xE8: 0x"));
    }
}
