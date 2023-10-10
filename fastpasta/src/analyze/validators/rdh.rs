//! contains the [RdhCruSanityValidator] that contains all the sanity checks for an [RDH].
//!
//! The [RdhCruSanityValidator] is composed of multiple subvalidators, each checking an [RDH] subword.

use crate::config::check::{ChecksOpt, System};
use crate::config::custom_checks::CustomChecksOpt;
use alice_protocol_reader::prelude::*;

use std::fmt::Write as _;

/// Enum to specialize the checks performed by the [RdhCruSanityValidator] for a specific system.
#[derive(Debug, Clone, Copy)]
pub enum SpecializeChecks {
    /// Specialize the checks for the Inner Tracking System.
    ITS,
}

/// Validator for the RDH CRU sanity checks.
pub struct RdhCruSanityValidator<T: RDH> {
    rdh0_validator: Rdh0Validator,
    rdh1_validator: &'static Rdh1Validator,
    rdh2_validator: &'static Rdh2Validator,
    rdh3_validator: &'static Rdh3Validator,
    _phantom: std::marker::PhantomData<T>,
    // valid_dataformat_reserved0: DataformatReserved,
    // valid link IDs are 0-11 and 15
    // datawrapper ID is 0 or 1
}

impl<T: RDH> Default for RdhCruSanityValidator<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Const values used by the RdhCrusanityValidator
const RDH1_VALIDATOR: Rdh1Validator = Rdh1Validator {
    valid_rdh1: Rdh1::const_default(),
};
const RDH2_VALIDATOR: Rdh2Validator = Rdh2Validator {};
const RDH3_VALIDATOR: Rdh3Validator = Rdh3Validator {};
const FEE_ID_SANITY_VALIDATOR: FeeIdSanityValidator = FeeIdSanityValidator::new((0, 6), (0, 47));

/// Specialized for ITS
const ITS_SYSTEM_ID: u8 = 32;
impl<T: RDH> RdhCruSanityValidator<T> {
    /// Creates a new [RdhCruSanityValidator] with default values.
    pub fn new() -> Self {
        Self {
            rdh0_validator: Rdh0Validator::default(),
            rdh1_validator: &RDH1_VALIDATOR,
            rdh2_validator: &RDH2_VALIDATOR,
            rdh3_validator: &RDH3_VALIDATOR,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Instantiate a [RdhCruSanityValidator] from a configuration object that implements [CustomChecksOpt] and [ChecksOpt].
    pub fn new_from_config(config: &'static (impl CustomChecksOpt + ChecksOpt)) -> Self {
        if config.custom_checks_enabled() {
            let mut validator = Self::with_custom_checks(config);
            if let Some(system) = config.check().unwrap().target() {
                match system {
                    System::ITS | System::ITS_Stave => {
                        validator.specialize(SpecializeChecks::ITS);
                    }
                }
            }
            validator
        } else if let Some(system) = config.check().unwrap().target() {
            match system {
                System::ITS | System::ITS_Stave => Self::with_specialization(SpecializeChecks::ITS),
            }
        } else {
            Self::default()
        }
    }

    /// Creates a new [RdhCruSanityValidator] specialized for a specific system.
    pub fn with_specialization(specialization: SpecializeChecks) -> Self {
        match specialization {
            SpecializeChecks::ITS => Self {
                rdh0_validator: Rdh0Validator::new(
                    None,
                    Rdh0::HEADER_SIZE,
                    FEE_ID_SANITY_VALIDATOR,
                    0,
                    Some(ITS_SYSTEM_ID),
                ),
                rdh1_validator: &RDH1_VALIDATOR,
                rdh2_validator: &RDH2_VALIDATOR,
                rdh3_validator: &RDH3_VALIDATOR,
                _phantom: std::marker::PhantomData,
            },
        }
    }

    /// Customize the RDH validator by supplying an instance that implements [CustomChecksOpt].
    /// If no custom checks are enabled that applies to the [RdhCruSanityValidator], the default instance is returned.
    fn with_custom_checks(custom_checks_opt: &'static impl CustomChecksOpt) -> Self {
        if let Some(rdh_version) = custom_checks_opt.rdh_version() {
            // New RDH0 validator
            Self {
                rdh0_validator: Rdh0Validator::new(
                    Some(rdh_version),
                    Rdh0::HEADER_SIZE,
                    FEE_ID_SANITY_VALIDATOR,
                    0,
                    None,
                ),
                rdh1_validator: &RDH1_VALIDATOR,
                rdh2_validator: &RDH2_VALIDATOR,
                rdh3_validator: &RDH3_VALIDATOR,
                _phantom: std::marker::PhantomData,
            }
        } else {
            Self::default()
        }
    }

    /// Specializes the [RdhCruSanityValidator] for a specific system.
    pub fn specialize(&mut self, specialization: SpecializeChecks) {
        match specialization {
            SpecializeChecks::ITS => {
                self.rdh0_validator.system_id = Some(ITS_SYSTEM_ID);
            }
        }
    }

    /// Performs the sanity checks on an [RDH].
    /// Returns [Ok] or an error type containing a [String] describing the error, if the sanity check failed.
    #[inline]
    pub fn sanity_check(&mut self, rdh: &T) -> Result<(), String> {
        let mut err_str = String::new();

        if let Err(e) = self.rdh0_validator.sanity_check(rdh.rdh0()) {
            err_str.push_str(&e);
        };
        if let Err(e) = self.rdh1_validator.sanity_check(rdh.rdh1()) {
            err_str.push_str(&e);
        };
        if let Err(e) = self.rdh2_validator.sanity_check(rdh.rdh2()) {
            err_str.push_str(&e);
        };
        if let Err(e) = self.rdh3_validator.sanity_check(rdh.rdh3()) {
            err_str.push_str(&e);
        };

        if rdh.dw() > 1 {
            let tmp = rdh.dw();
            write!(err_str, "dw = {:#x} ", tmp).unwrap();
        }
        if rdh.data_format() > 2 {
            let tmp = rdh.data_format();
            write!(err_str, "data format = {:#x} ", tmp).unwrap();
        }

        if !err_str.is_empty() {
            return Err(format!("[E10] RDH sanity check failed: {err_str}"));
        }

        Ok(())
    }
}
struct FeeIdSanityValidator {
    layer_min_max: (u8, u8),
    stave_number_min_max: (u8, u8),
}

impl FeeIdSanityValidator {
    const fn new(layer_min_max: (u8, u8), stave_number_min_max: (u8, u8)) -> Self {
        if layer_min_max.0 > layer_min_max.1 {
            panic!("Layer min must be smaller than layer max");
        }
        if stave_number_min_max.0 > stave_number_min_max.1 {
            panic!("Stave number min must be smaller than stave number max");
        }
        Self {
            layer_min_max,
            stave_number_min_max,
        }
    }
    fn sanity_check(&self, fee_id: FeeId) -> Result<(), String> {
        // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number
        // 5:0 stave number
        // 7:6 reserved
        // 9:8 fiber uplink
        // 11:10 reserved
        // 14:12 layer
        // 15 reserved

        let mut err_str = String::new();

        // Extract mask over reserved bits and check if it is 0
        let reserved_bits_mask: u16 = 0b1000_1100_1100_0000;
        let reserved_bits = fee_id.0 & reserved_bits_mask;
        if reserved_bits != 0 {
            write!(err_str, "reserved bits = {:#x} ", reserved_bits).unwrap();
        }
        // Extract stave_number from 6 LSB [5:0]
        let stave_number = crate::words::its::stave_number_from_feeid(fee_id.0);
        if stave_number < self.stave_number_min_max.0 || stave_number > self.stave_number_min_max.1
        {
            write!(err_str, "stave number = {} ", stave_number).unwrap();
        }

        // Extract layer from 3 bits [14:12]
        let layer = crate::words::its::layer_from_feeid(fee_id.0);

        if layer < self.layer_min_max.0 || layer > self.layer_min_max.1 {
            write!(err_str, "layer = {} ", layer).unwrap();
        }

        if !err_str.is_empty() {
            return Err(err_str.to_owned());
        }

        Ok(())
    }
}

/// Validator for individual [Rdh0] RDH subwords. Performs a basic sanity check.
pub struct Rdh0Validator {
    header_id: Option<u8>, // The first Rdh0 checked will determine what is a valid header_id
    header_size: u8,
    fee_id: FeeIdSanityValidator,
    priority_bit: u8,
    system_id: Option<u8>,
    reserved0: u16,
}

impl Default for Rdh0Validator {
    fn default() -> Self {
        Self::new(None, Rdh0::HEADER_SIZE, FEE_ID_SANITY_VALIDATOR, 0, None)
    }
}

impl Rdh0Validator {
    fn new(
        header_id: Option<u8>,
        header_size: u8,
        fee_id: FeeIdSanityValidator,
        priority_bit: u8,
        system_id: Option<u8>,
    ) -> Self {
        Self {
            header_id,
            header_size,
            fee_id,
            priority_bit,
            system_id,
            reserved0: 0,
        }
    }

    /// Check consistency of a [Rdh0] RDH subword
    pub fn sanity_check(&mut self, rdh0: &Rdh0) -> Result<(), String> {
        if self.header_id.is_none() {
            self.header_id = Some(rdh0.header_id);
        }
        let mut err_str = String::new();
        if rdh0.header_id != self.header_id.unwrap() {
            write!(
                err_str,
                "Header ID = {} (expected {})",
                rdh0.header_id,
                self.header_id.unwrap()
            )
            .unwrap();
        }
        if rdh0.header_size != self.header_size {
            write!(err_str, "Header size = {:#x} ", rdh0.header_size).unwrap();
        }
        if let Err(e) = self.fee_id.sanity_check(FeeId(rdh0.fee_id())) {
            write!(err_str, "FEE ID = [{}] ", e).unwrap();
        }
        if rdh0.priority_bit != self.priority_bit {
            write!(err_str, "Priority bit = {:#x} ", rdh0.priority_bit).unwrap();
        }
        if let Some(valid_system_id) = self.system_id {
            if rdh0.system_id != valid_system_id {
                write!(err_str, "system_id = {:#x} ", rdh0.system_id).unwrap();
            }
        }

        if rdh0.reserved0 != self.reserved0 {
            let tmp = rdh0.reserved0;
            write!(err_str, "reserved0 = {tmp:#x} ").unwrap();
        }
        if !err_str.is_empty() {
            return Err(format!("RDH0: {err_str}"));
        }
        Ok(())
    }
}

/// Validator for the [RDH] subword [RDH1][Rdh1].
#[derive(Default)]
struct Rdh1Validator {
    valid_rdh1: Rdh1,
}
impl Rdh1Validator {
    pub fn sanity_check(&self, rdh1: &Rdh1) -> Result<(), String> {
        let mut err_str = String::new();
        if rdh1.reserved0() != self.valid_rdh1.reserved0() {
            write!(err_str, "reserved0 = {:#x} ", rdh1.reserved0()).unwrap();
        }
        // Max bunch counter is 0xdeb
        if rdh1.bc() > 0xdeb {
            write!(err_str, "BC = {:#x} ", rdh1.bc()).unwrap();
        }

        if !err_str.is_empty() {
            return Err(format!("RDH1: {err_str}"));
        }
        Ok(())
    }
}

#[derive(Default)]
struct Rdh2Validator;
impl Rdh2Validator {
    pub fn sanity_check(&self, rdh2: &Rdh2) -> Result<(), String> {
        let mut err_str = String::new();
        if rdh2.reserved0 != 0 {
            write!(err_str, "reserved0 = {:#x} ", rdh2.reserved0).unwrap();
        }

        if rdh2.stop_bit > 1 {
            write!(err_str, "stop bit = {:#x} ", rdh2.stop_bit).unwrap();
        }
        let spare_bits_15_to_26_set: u32 = 0b0000_0111_1111_1111_1000_0000_0000_0000;
        if rdh2.trigger_type == 0 || (rdh2.trigger_type & spare_bits_15_to_26_set != 0) {
            let tmp = rdh2.trigger_type;
            write!(err_str, "Spare bits set in trigger_type = {tmp:#x} ").unwrap();
        }

        if !err_str.is_empty() {
            return Err(format!("RDH2: {err_str}"));
        }
        Ok(())
    }
}

#[derive(Default)]
struct Rdh3Validator;
impl Rdh3Validator {
    pub fn sanity_check(&self, rdh3: &Rdh3) -> Result<(), String> {
        let mut err_str = String::new();

        if rdh3.reserved0 != 0 {
            let tmp = rdh3.reserved0;
            write!(err_str, "reserved0 = {:#x} ", tmp).unwrap();
        }

        // Updated for v1.21.0 (bit 4:11 are no longer reserved)
        let reserved_bits_12_to_23_set: u32 = 0b1111_1111_1111_0000_0000_0000;
        if rdh3.detector_field & reserved_bits_12_to_23_set != 0 {
            let tmp = rdh3.detector_field;
            write!(err_str, "detector_field = {:#x} ", tmp).unwrap();
        }

        /***  New from v1.21.0  ***
        (not checked in sanity check as both set and not set is valid)

        - 11:6: User configurable reserved | can be set to configurable value by register
        - 5: Stave autorecovery including trigger ramp | User configurable: To be set manually during the autorecovery including a trigger ramping
        - 4: Trigger ramp | User configurable: To be set manually during trigger ramping  */

        // No checks on Par bit

        if !err_str.is_empty() {
            return Err(format!("RDH3: {err_str}"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alice_protocol_reader::prelude::test_data::{CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V7};

    const _CORRECT_RDH0: Rdh0 = Rdh0::new(7, Rdh0::HEADER_SIZE, FeeId(20522), 0, ITS_SYSTEM_ID, 0);
    const CORRECT_RDH1: Rdh1 = Rdh1::new(BcReserved(0), 0);
    const CORRECT_RDH2: Rdh2 = Rdh2::new(27139, 0, 0, 0);
    const CORRECT_RDH3: Rdh3 = Rdh3::new(0, 0, 0);

    #[test]
    fn validate_fee_id() {
        let validator = FEE_ID_SANITY_VALIDATOR;
        let fee_id = FeeId(0x502A);
        assert!(validator.sanity_check(fee_id).is_ok());
    }

    #[test]
    fn invalidate_fee_id_bad_reserved() {
        let validator = FEE_ID_SANITY_VALIDATOR;
        let fee_id_bad_reserved0 = FeeId(0b1000_0000_0000_0000);
        let fee_id_bad_reserved1 = FeeId(0b0000_0100_0000_0000);
        let fee_id_bad_reserved2 = FeeId(0b0000_0000_0100_0000);
        let res0 = validator.sanity_check(fee_id_bad_reserved0);
        println!("{res0:?} ");
        let res1 = validator.sanity_check(fee_id_bad_reserved1);
        println!("{res1:?} ");
        let res2 = validator.sanity_check(fee_id_bad_reserved2);
        println!("{res2:?} `");
        assert!(validator.sanity_check(fee_id_bad_reserved0).is_err());
        assert!(validator.sanity_check(fee_id_bad_reserved1).is_err());
        assert!(validator.sanity_check(fee_id_bad_reserved2).is_err());
    }
    #[test]
    fn invalidate_fee_id_bad_layer() {
        let validator = FEE_ID_SANITY_VALIDATOR;
        let fee_id_invalid_layer_is_7 = FeeId(0b0111_0000_0000_0000);
        let res = validator.sanity_check(fee_id_invalid_layer_is_7);
        println!("{res:?}");
        assert!(validator.sanity_check(fee_id_invalid_layer_is_7).is_err());
    }

    #[test]
    fn invalidate_fee_id_bad_stave_number() {
        let validator = FEE_ID_SANITY_VALIDATOR;
        let fee_id_bad_stave_number_is_48 = FeeId(0x30);
        let res = validator.sanity_check(fee_id_bad_stave_number_is_48);
        println!("{res:?}");
        assert!(res.is_err());
    }
    // RDH0 sanity check
    #[test]
    fn validate_rdh0() {
        let mut validator = Rdh0Validator::default();
        let rdh0 = Rdh0::new(7, Rdh0::HEADER_SIZE, FeeId(0x502A), 0, ITS_SYSTEM_ID, 0);
        let rdh0_2 = Rdh0::new(7, Rdh0::HEADER_SIZE, FeeId(0x502A), 0, ITS_SYSTEM_ID, 0);

        let res0 = validator.sanity_check(&rdh0);
        assert!(res0.is_ok());
        let res1 = validator.sanity_check(&rdh0_2);
        assert!(res1.is_ok());
    }
    #[test]
    fn invalidate_rdh0_bad_header_id() {
        let mut validator = Rdh0Validator::new(
            Some(7),
            0x40,
            FeeIdSanityValidator {
                layer_min_max: (0, 7),
                stave_number_min_max: (0, 47),
            },
            0,
            Some(ITS_SYSTEM_ID),
        );
        let rdh0 = Rdh0::new(0x7, 0x40, FeeId(0x502A), 0, ITS_SYSTEM_ID, 0);

        let res = validator.sanity_check(&rdh0);
        assert!(
            res.is_ok(),
            "Invalidated a correct RDH0: {}",
            res.unwrap_err()
        );
        // Change to different header_id
        let rdh0_new = Rdh0::new(0x8, 0x40, FeeId(0x502A), 0, ITS_SYSTEM_ID, 0);

        assert!(validator.sanity_check(&rdh0_new).is_err());
    }
    #[test]
    fn invalidate_rdh0_bad_header_size() {
        let mut validator = Rdh0Validator::default();
        let rdh0 = Rdh0::new(7, 3, FeeId(0x502A), 0, ITS_SYSTEM_ID, 0);
        let res = validator.sanity_check(&rdh0);
        println!("{res:?}");
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh0_bad_fee_id() {
        let mut validator = Rdh0Validator::default();
        let fee_id_bad_stave_number_is_48 = FeeId(0x30);
        let rdh0 = Rdh0::new(
            7,
            Rdh0::HEADER_SIZE,
            fee_id_bad_stave_number_is_48,
            0,
            ITS_SYSTEM_ID,
            0,
        );

        let res = validator.sanity_check(&rdh0);
        println!("{res:?}");
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh0_bad_system_id() {
        let mut validator = Rdh0Validator::new(
            None,
            Rdh0::HEADER_SIZE,
            FEE_ID_SANITY_VALIDATOR,
            0,
            Some(ITS_SYSTEM_ID),
        );
        let rdh0 = Rdh0::new(7, Rdh0::HEADER_SIZE, FeeId(0x502A), 0, 3, 0);

        let res = validator.sanity_check(&rdh0);
        println!("{res:?}");
        assert!(res.is_err());
    }

    #[test]
    fn validate_rdh0_non_its_system_id() {
        let mut validator =
            Rdh0Validator::new(None, Rdh0::HEADER_SIZE, FEE_ID_SANITY_VALIDATOR, 0, None);

        let rdh0 = Rdh0::new(7, Rdh0::HEADER_SIZE, FeeId(0x502A), 0, 0x99, 0);

        let res = validator.sanity_check(&rdh0);
        println!("{res:?}");
        assert!(res.is_ok());
    }

    #[test]
    fn invalidate_rdh0_bad_reserved0() {
        let mut validator = Rdh0Validator::new(
            None,
            Rdh0::HEADER_SIZE,
            FEE_ID_SANITY_VALIDATOR,
            0,
            Some(ITS_SYSTEM_ID),
        );
        let rdh0 = Rdh0::new(7, Rdh0::HEADER_SIZE, FeeId(0x502A), 0, ITS_SYSTEM_ID, 0x3);

        let res = validator.sanity_check(&rdh0);
        println!("{res:?}");
        assert!(res.is_err());
    }

    // RDH1 sanity check
    #[test]
    fn validate_rdh1() {
        let validator = RDH1_VALIDATOR;
        let rdh1 = Rdh1::new(BcReserved(0), 0);
        let res = validator.sanity_check(&rdh1);
        assert!(res.is_ok());
    }
    #[test]
    fn invalidate_rdh1_bad_reserved0() {
        let validator = RDH1_VALIDATOR;
        let rdh1 = Rdh1::new(BcReserved(1 << 12), 0);

        let res = validator.sanity_check(&rdh1);
        println!("{res:?}");
        assert!(res.is_err());
    }

    // RDH2 sanity check
    #[test]
    fn validate_rdh2() {
        let validator = RDH2_VALIDATOR;
        let rdh2 = Rdh2 {
            trigger_type: 1,
            pages_counter: 0,
            stop_bit: 0,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh2);
        assert!(res.is_ok());
    }
    #[test]
    fn invalidate_rdh2_bad_reserved0() {
        let validator = RDH2_VALIDATOR;
        let rdh2 = Rdh2 {
            trigger_type: 1,
            pages_counter: 0,
            stop_bit: 0,
            reserved0: 1,
        };
        let res = validator.sanity_check(&rdh2);
        println!("{res:?}");
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh2_bad_trigger_type() {
        let validator = RDH2_VALIDATOR;
        let rdh2 = Rdh2 {
            trigger_type: 0,
            pages_counter: 0,
            stop_bit: 0,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh2);
        println!("{res:?}");
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh2_bad_stop_bit() {
        let validator = RDH2_VALIDATOR;
        let rdh2 = Rdh2 {
            trigger_type: 1,
            pages_counter: 0,
            stop_bit: 2,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh2);
        println!("{res:?}");
        assert!(res.is_err());
    }

    // RDH3 sanity check
    #[test]
    fn validate_rdh3() {
        let validator = RDH3_VALIDATOR;
        let rdh3 = Rdh3 {
            detector_field: 0,
            par_bit: 0,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh3);
        assert!(res.is_ok());
    }
    #[test]
    fn invalidate_rdh3_bad_reserved0() {
        let validator = RDH3_VALIDATOR;
        let rdh3 = Rdh3 {
            detector_field: 0,
            par_bit: 0,
            reserved0: 1,
        };
        let res = validator.sanity_check(&rdh3);
        println!("{res:?}");
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh3_bad_detector_field() {
        let validator = RDH3_VALIDATOR;
        let _reserved_bits_12_to_23_set: u32 = 0b1111_1111_1111_0000_0000_0000; // Updated for v1.21.0
        let example_bad_detector_field = 0b1000_0000_0000_0000;
        let rdh3 = Rdh3 {
            detector_field: example_bad_detector_field,
            par_bit: 0,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh3);
        println!("{res:?}");
        assert!(res.is_err());
    }

    #[test]
    fn validate_rdh_cru_v7() {
        let mut validator = RdhCruSanityValidator::new();
        validator.rdh1_validator = &RDH1_VALIDATOR;
        let res = validator.sanity_check(&CORRECT_RDH_CRU_V7);
        assert!(res.is_ok());
    }
    #[test]
    fn invalidate_rdh_cru_v7_bad_header_id() {
        let mut validator = RdhCruSanityValidator::default();
        let rdh0 = CORRECT_RDH_CRU_V7.rdh0();
        let rdh_cru: RdhCru = RdhCru::new(
            *rdh0,
            CORRECT_RDH_CRU_V7.offset_to_next(),
            CORRECT_RDH_CRU_V7.payload_size(),
            CORRECT_RDH_CRU_V7.link_id(),
            CORRECT_RDH_CRU_V7.packet_counter(),
            CruidDw(CORRECT_RDH_CRU_V7.cru_id()),
            CORRECT_RDH1,
            DataformatReserved(2),
            CORRECT_RDH2,
            CORRECT_RDH_CRU_V7.reserved1(),
            CORRECT_RDH3,
            CORRECT_RDH_CRU_V7.reserved2(),
        );
        assert!(validator.sanity_check(&rdh_cru).is_ok());
        let rdh_cru_bad_header: RdhCru = RdhCru::new(
            Rdh0::new(0, 0, FeeId(9), 0, 20, 0),
            CORRECT_RDH_CRU_V7.offset_to_next(),
            CORRECT_RDH_CRU_V7.payload_size(),
            CORRECT_RDH_CRU_V7.link_id(),
            CORRECT_RDH_CRU_V7.packet_counter(),
            CruidDw(CORRECT_RDH_CRU_V7.cru_id()),
            CORRECT_RDH1,
            DataformatReserved(2),
            CORRECT_RDH2,
            CORRECT_RDH_CRU_V7.reserved1(),
            CORRECT_RDH3,
            CORRECT_RDH_CRU_V7.reserved2(),
        );
        let res = validator.sanity_check(&rdh_cru_bad_header);
        println!("{res:?}");
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh_cru_v7_multiple_errors() {
        let mut validator = RdhCruSanityValidator::default();
        let rdh_cru_bad_fields: RdhCru = RdhCru::new(
            Rdh0::new(0, 0, FeeId(9), 0, 20, 0),
            CORRECT_RDH_CRU_V7.offset_to_next(),
            CORRECT_RDH_CRU_V7.payload_size(),
            CORRECT_RDH_CRU_V7.link_id(),
            CORRECT_RDH_CRU_V7.packet_counter(),
            CruidDw(CORRECT_RDH_CRU_V7.cru_id()),
            Rdh1::new(BcReserved(1), 10),
            DataformatReserved(2),
            CORRECT_RDH2,
            CORRECT_RDH_CRU_V7.reserved1(),
            Rdh3::new(5, 5, 5),
            CORRECT_RDH_CRU_V7.reserved2(),
        );
        let res = validator.sanity_check(&rdh_cru_bad_fields);
        println!("{res:?}");
        assert!(res.is_err(), "{rdh_cru_bad_fields}");
    }

    #[test]
    fn allow_rdh_cru_v7_non_its_system_id() {
        let mut validator = RdhCruSanityValidator::default();
        let non_its_system_id = 0x99;
        let rdh_cru: RdhCru = RdhCru::new(
            Rdh0::new(7, 0x40, FeeId(0), 0, non_its_system_id, 0),
            CORRECT_RDH_CRU_V7.offset_to_next(),
            CORRECT_RDH_CRU_V7.payload_size(),
            CORRECT_RDH_CRU_V7.link_id(),
            CORRECT_RDH_CRU_V7.packet_counter(),
            CruidDw(CORRECT_RDH_CRU_V7.cru_id()),
            CORRECT_RDH1,
            DataformatReserved(2),
            CORRECT_RDH2,
            CORRECT_RDH_CRU_V7.reserved1(),
            CORRECT_RDH3,
            CORRECT_RDH_CRU_V7.reserved2(),
        );

        let res = validator.sanity_check(&rdh_cru);
        println!("{res:?}");
        assert!(res.is_ok());
    }

    #[test]
    fn invalidate_rdh_cru_v7_its_specialized_bad_system_id() {
        let mut validator = RdhCruSanityValidator::default();
        validator.specialize(SpecializeChecks::ITS);
        let non_its_system_id = 0x99;
        let rdh_cru: RdhCru = RdhCru::new(
            Rdh0::new(7, 0x40, FeeId(0), 0, non_its_system_id, 0),
            CORRECT_RDH_CRU_V7.offset_to_next(),
            CORRECT_RDH_CRU_V7.payload_size(),
            CORRECT_RDH_CRU_V7.link_id(),
            CORRECT_RDH_CRU_V7.packet_counter(),
            CruidDw(CORRECT_RDH_CRU_V7.cru_id()),
            CORRECT_RDH1,
            DataformatReserved(2),
            CORRECT_RDH2,
            CORRECT_RDH_CRU_V7.reserved1(),
            CORRECT_RDH3,
            CORRECT_RDH_CRU_V7.reserved2(),
        );

        let res = validator.sanity_check(&rdh_cru);
        println!("{res:?}");
        assert!(res.is_err());
    }

    #[test]
    fn validate_rdh_cru_v6() {
        let mut validator = RdhCruSanityValidator::default();
        let res = validator.sanity_check(&CORRECT_RDH_CRU_V6);
        assert!(res.is_ok());
    }
    #[test]
    fn invalidate_rdh_cru_v6_bad_header_id() {
        let mut validator = RdhCruSanityValidator::default();
        let non_its_system_id = 0x90;
        let rdh_cru: RdhCru = RdhCru::new(
            Rdh0::new(6, 0x40, FeeId(0), 0, non_its_system_id, 0),
            CORRECT_RDH_CRU_V7.offset_to_next(),
            CORRECT_RDH_CRU_V7.payload_size(),
            CORRECT_RDH_CRU_V7.link_id(),
            CORRECT_RDH_CRU_V7.packet_counter(),
            CruidDw(CORRECT_RDH_CRU_V7.cru_id()),
            CORRECT_RDH1,
            DataformatReserved(2),
            CORRECT_RDH2,
            CORRECT_RDH_CRU_V7.reserved1(),
            CORRECT_RDH3,
            CORRECT_RDH_CRU_V7.reserved2(),
        );
        let res = validator.sanity_check(&rdh_cru);
        println!("{res:?}");
        assert!(res.is_ok());
        let rdh_cru_bad_header_id: RdhCru = RdhCru::new(
            Rdh0::new(1, 0x40, FeeId(0), 0, non_its_system_id, 0),
            CORRECT_RDH_CRU_V7.offset_to_next(),
            CORRECT_RDH_CRU_V7.payload_size(),
            CORRECT_RDH_CRU_V7.link_id(),
            CORRECT_RDH_CRU_V7.packet_counter(),
            CruidDw(CORRECT_RDH_CRU_V7.cru_id()),
            CORRECT_RDH1,
            DataformatReserved(2),
            CORRECT_RDH2,
            CORRECT_RDH_CRU_V7.reserved1(),
            CORRECT_RDH3,
            CORRECT_RDH_CRU_V7.reserved2(),
        );
        let res_new = validator.sanity_check(&rdh_cru_bad_header_id);
        assert!(res_new.is_err(), "{rdh_cru_bad_header_id}");
    }
    #[test]
    fn invalidate_rdh_cru_v6_multiple_errors() {
        let mut validator = RdhCruSanityValidator::default();
        let rdh_cru: RdhCru = RdhCru::new(
            Rdh0::new(6, 0, FeeId(0), 0, 0, 0),
            CORRECT_RDH_CRU_V7.offset_to_next(),
            CORRECT_RDH_CRU_V7.payload_size(),
            CORRECT_RDH_CRU_V7.link_id(),
            CORRECT_RDH_CRU_V7.packet_counter(),
            CruidDw(CORRECT_RDH_CRU_V7.cru_id()),
            CORRECT_RDH1,
            DataformatReserved(2),
            CORRECT_RDH2,
            1,
            Rdh3::new(5, 99, 9),
            1,
        );
        let res = validator.sanity_check(&rdh_cru);
        println!("{res:?}");
        assert!(res.is_err());
    }
}
