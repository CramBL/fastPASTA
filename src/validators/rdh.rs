use crate::words::lib::RDH;
use crate::words::rdh::{FeeId, Rdh0, Rdh1, Rdh2, Rdh3};
use std::fmt::Write as _;

pub struct FeeIdSanityValidator {
    pub reserved0: u8,
    layer_min_max: (u8, u8),
    pub reserved1: u8,
    pub reserved2: u8,
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
            reserved0: 0,
            layer_min_max,
            reserved1: 0,
            reserved2: 0,
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
        let mut err_cnt: u8 = 0;

        // Extract mask over reserved bits and check if it is 0
        let reserved_bits_mask: u16 = 0b1000_1100_1100_0000;
        let reserved_bits = fee_id.0 & reserved_bits_mask;
        if reserved_bits != 0 {
            err_cnt += 1;
            write!(
                err_str,
                "{} = {:#x} ",
                stringify!(reserved_bits),
                reserved_bits
            )
            .unwrap();
        }
        // Extract stave_number from 6 LSB [5:0]
        let stave_number = crate::words::rdh::stave_number_from_feeid(fee_id.0);
        if stave_number < self.stave_number_min_max.0 || stave_number > self.stave_number_min_max.1
        {
            err_cnt += 1;
            write!(err_str, "{} = {} ", stringify!(stave_number), stave_number).unwrap();
        }

        // Extract layer from 3 bits [14:12]
        let layer = crate::words::rdh::layer_from_feeid(fee_id.0);

        if layer < self.layer_min_max.0 || layer > self.layer_min_max.1 {
            err_cnt += 1;
            write!(err_str, "{} = {} ", stringify!(layer), layer).unwrap();
        }

        if err_cnt != 0 {
            return Err(err_str.to_owned());
        }

        Ok(())
    }
}

const FEE_ID_SANITY_VALIDATOR: FeeIdSanityValidator = FeeIdSanityValidator::new((0, 6), (0, 47));

pub struct Rdh0Validator {
    pub header_id: Option<u8>, // The first Rdh0 checked will determine what is a valid header_id
    pub header_size: u8,
    pub fee_id: FeeIdSanityValidator,
    pub priority_bit: u8,
    pub system_id: u8,
    pub reserved0: u16,
}
impl Default for Rdh0Validator {
    fn default() -> Self {
        Self::new()
    }
}

const ITS_SYSTEM_ID: u8 = 32;
impl Rdh0Validator {
    pub fn new() -> Self {
        Self {
            header_id: None,
            header_size: 0x40,
            fee_id: FEE_ID_SANITY_VALIDATOR,
            priority_bit: 0,
            system_id: ITS_SYSTEM_ID,
            reserved0: 0,
        }
    }
    pub fn sanity_check(&mut self, rdh0: &Rdh0) -> Result<(), String> {
        if self.header_id.is_none() {
            self.header_id = Some(rdh0.header_id);
        }
        let mut err_str = String::new();
        let mut err_cnt: u8 = 0;
        if rdh0.header_id != self.header_id.unwrap() {
            err_cnt += 1;
            write!(
                err_str,
                "{} = {:#x} ",
                stringify!(header_id),
                rdh0.header_id
            )
            .unwrap();
        }
        if rdh0.header_size != self.header_size {
            err_cnt += 1;
            write!(
                err_str,
                "{} = {:#x} ",
                stringify!(header_size),
                rdh0.header_size
            )
            .unwrap();
        }
        match self.fee_id.sanity_check(rdh0.fee_id) {
            Ok(_) => {} // Check passed
            Err(e) => {
                err_cnt += 1;
                write!(err_str, "{} = {} ", stringify!(fee_id), e).unwrap();
            }
        }
        if rdh0.priority_bit != self.priority_bit {
            err_cnt += 1;
            write!(
                err_str,
                "{} = {:#x} ",
                stringify!(priority_bit),
                rdh0.priority_bit
            )
            .unwrap();
        }
        if rdh0.system_id != self.system_id {
            err_cnt += 1;
            write!(
                err_str,
                "{} = {:#x} ",
                stringify!(system_id),
                rdh0.system_id
            )
            .unwrap();
        }
        if rdh0.reserved0 != self.reserved0 {
            err_cnt += 1;
            let tmp = rdh0.reserved0;
            write!(err_str, "{} = {:#x} ", stringify!(rdh0.reserved0), tmp).unwrap();
        }
        if err_cnt != 0 {
            return Err(err_str.to_owned());
        }
        Ok(())
    }
}

pub struct Rdh1Validator {
    valid_rdh1: Rdh1,
}
impl Rdh1Validator {
    pub fn sanity_check(&self, rdh1: &Rdh1) -> Result<(), String> {
        let mut err_str = String::new();
        let mut err_cnt: u8 = 0;
        if rdh1.reserved0() != self.valid_rdh1.reserved0() {
            err_cnt += 1;
            write!(
                err_str,
                "{} = {:#x} ",
                stringify!(rdh1.reserved0),
                rdh1.reserved0()
            )
            .unwrap();
        }
        // Max bunch counter is 0xdeb
        if rdh1.bc() > 0xdeb {
            err_cnt += 1;
            write!(err_str, "{} = {:#x} ", stringify!(bc), rdh1.bc()).unwrap();
        }

        if err_cnt != 0 {
            return Err(err_str.to_owned());
        }
        Ok(())
    }
}
pub const RDH1_VALIDATOR: Rdh1Validator = Rdh1Validator {
    valid_rdh1: Rdh1::test_new(0, 0, 0),
};

pub struct Rdh2Validator;
impl Rdh2Validator {
    pub fn sanity_check(&self, rdh2: &Rdh2) -> Result<(), String> {
        let mut err_str = String::new();
        let mut err_cnt: u8 = 0;
        if rdh2.reserved0 != 0 {
            err_cnt += 1;
            write!(
                err_str,
                "{} = {:#x} ",
                stringify!(rdh2.reserved0),
                rdh2.reserved0
            )
            .unwrap();
        }

        if rdh2.stop_bit > 1 {
            err_cnt += 1;
            write!(err_str, "{} = {:#x} ", stringify!(stop_bit), rdh2.stop_bit).unwrap();
        }
        let spare_bits_15_to_26_set: u32 = 0b0000_0111_1111_1111_1000_0000_0000_0000;
        if rdh2.trigger_type == 0 || (rdh2.trigger_type & spare_bits_15_to_26_set != 0) {
            err_cnt += 1;
            let tmp = rdh2.trigger_type;
            write!(err_str, "{} = {:#x} ", stringify!(trigger_type), tmp).unwrap();
        }

        if err_cnt != 0 {
            return Err(err_str.to_owned());
        }
        Ok(())
    }
}

pub const RDH2_VALIDATOR: Rdh2Validator = Rdh2Validator {};

pub struct Rdh3Validator;
impl Rdh3Validator {
    pub fn sanity_check(&self, rdh3: &Rdh3) -> Result<(), String> {
        let mut err_str = String::new();
        let mut err_cnt: u8 = 0;
        if rdh3.reserved0 != 0 {
            err_cnt += 1;
            let tmp = rdh3.reserved0;
            write!(err_str, "{} = {:#x} ", stringify!(rdh3.reserved0), tmp).unwrap();
        }
        let reserved_bits_4_to_23_set: u32 = 0b1111_1111_1111_1111_1111_0000;
        if rdh3.detector_field & reserved_bits_4_to_23_set != 0 {
            err_cnt += 1;
            let tmp = rdh3.detector_field;
            write!(err_str, "{} = {:#x} ", stringify!(detector_field), tmp).unwrap();
        }

        // No checks on Par bit

        if err_cnt != 0 {
            return Err(err_str.to_owned());
        }
        Ok(())
    }
}

pub const RDH3_VALIDATOR: Rdh3Validator = Rdh3Validator {};

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

impl<T: RDH> RdhCruSanityValidator<T> {
    pub fn new() -> Self {
        Self {
            rdh0_validator: Rdh0Validator::default(),
            rdh1_validator: &RDH1_VALIDATOR,
            rdh2_validator: &RDH2_VALIDATOR,
            rdh3_validator: &RDH3_VALIDATOR,
            _phantom: std::marker::PhantomData,
        }
    }
    #[inline]
    pub fn sanity_check(&mut self, rdh: &T) -> Result<(), String> {
        let mut err_str = String::from("RDH sanity check failed: ");
        let mut err_cnt: u8 = 0;
        let mut rdh_errors: Vec<String> = vec![];
        match self.rdh0_validator.sanity_check(rdh.rdh0()) {
            Ok(_) => (),
            Err(e) => {
                err_cnt += 1;
                rdh_errors.push(e);
            }
        };
        match self.rdh1_validator.sanity_check(rdh.rdh1()) {
            Ok(_) => (),
            Err(e) => {
                err_cnt += 1;
                rdh_errors.push(e);
            }
        };
        match self.rdh2_validator.sanity_check(rdh.rdh2()) {
            Ok(_) => (),
            Err(e) => {
                err_cnt += 1;
                rdh_errors.push(e);
            }
        };
        match self.rdh3_validator.sanity_check(rdh.rdh3()) {
            Ok(_) => (),
            Err(e) => {
                err_cnt += 1;
                rdh_errors.push(e);
            }
        };

        if rdh.dw() > 1 {
            err_cnt += 1;
            let tmp = rdh.dw();
            write!(err_str, "{} = {:#x} ", stringify!(dw), tmp).unwrap();
        }
        if rdh.data_format() > 2 {
            err_cnt += 1;
            let tmp = rdh.data_format();
            write!(err_str, "{} = {:#x} ", stringify!(data_format), tmp).unwrap();
        }

        rdh_errors.into_iter().for_each(|e| {
            err_str.push_str(&e);
        });

        if err_cnt != 0 {
            return Err(err_str.to_owned());
        }

        Ok(())
    }
}

pub struct RdhCruRunningChecker<T: RDH> {
    pub expect_pages_counter: u16,
    pub last_rdh2: Option<Rdh2>,
    // The first 2 RDHs are used to determine what the expected page counter increments are
    first_rdh_cru: Option<T>,
    second_rdh_cru: Option<T>,
    expect_pages_counter_increment: u16,
}

impl<T: RDH> Default for RdhCruRunningChecker<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RDH> RdhCruRunningChecker<T> {
    pub fn new() -> Self {
        Self {
            expect_pages_counter: 0,
            last_rdh2: None,
            first_rdh_cru: None,
            second_rdh_cru: None,
            expect_pages_counter_increment: 1,
        }
    }
    #[inline]
    pub fn check(&mut self, rdh: &T) -> Result<(), String> {
        if self.first_rdh_cru.is_none() {
            self.first_rdh_cru = Some(T::load(&mut rdh.to_byte_slice()).unwrap());
        } else if self.second_rdh_cru.is_none() {
            self.second_rdh_cru = Some(T::load(&mut rdh.to_byte_slice()).unwrap());
            self.expect_pages_counter_increment =
                self.second_rdh_cru.as_ref().unwrap().rdh2().pages_counter;
        }

        let mut err_str = String::new();
        let mut rdh_errors: Vec<String> = vec![];
        let mut err_cnt: u8 = 0;

        match self.check_stop_bit_and_page_counter(rdh.rdh2()) {
            Ok(_) => (),
            Err(e) => {
                err_cnt += 1;
                rdh_errors.push(e);
            }
        };

        if err_cnt != 0 {
            rdh_errors.into_iter().for_each(|e| {
                err_str.push_str(&e);
            });
            return Err(err_str);
        }

        self.last_rdh2 = Some(*rdh.rdh2());

        Ok(())
    }

    /// # Check `stop_bit` and `pages_counter` across a CDP
    ///
    /// 1. If `stop_bit` is 0, page counter should be equal to either:
    ///      * the previous `pages_counter` + 1
    ///      * 0 if the `stop_bit` of the previous `Rdh2` was 1 (handled in the match on `stop_bit == 1`)
    ///
    ///     Side effect: `self.expect_pages_counter += 1`
    ///
    /// 2. If `stop_bit` is 1, `pages_counter` should be equal to the previous `pages_counter` + 1
    ///
    ///    Side effect: `self.expect_pages_counter = 0`
    #[inline]
    fn check_stop_bit_and_page_counter(&mut self, rdh2: &Rdh2) -> Result<(), String> {
        let mut err_str = String::from("RDH2 check stop_bit & page_counter failed: ");
        let mut err_cnt: u8 = 0;
        match rdh2.stop_bit {
            0 => {
                if rdh2.pages_counter != self.expect_pages_counter {
                    err_cnt += 1;
                    let tmp = rdh2.pages_counter;
                    write!(
                        err_str,
                        "pages_counter = {tmp} expected: {}",
                        self.expect_pages_counter
                    )
                    .unwrap();
                }
                self.expect_pages_counter += self.expect_pages_counter_increment;
            }
            1 => {
                if rdh2.pages_counter != self.expect_pages_counter {
                    err_cnt += 1;
                    let tmp = rdh2.pages_counter;
                    write!(
                        err_str,
                        "pages_counter = {tmp} expected: {}",
                        self.expect_pages_counter
                    )
                    .unwrap();
                }
                self.expect_pages_counter = 0;
            }
            _ => {
                err_cnt += 1;
                let tmp = rdh2.stop_bit;
                write!(err_str, "stop_bit = {tmp}").unwrap();
            }
        };

        if err_cnt != 0 {
            return Err(err_str.to_owned());
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::words::rdh_cru::test_data::{CORRECT_RDH_CRU_V6, CORRECT_RDH_CRU_V7};

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
        let res = validator.sanity_check(fee_id_bad_reserved0);
        println!("{res:?} ");
        let res = validator.sanity_check(fee_id_bad_reserved1);
        println!("{res:?} ");
        let res = validator.sanity_check(fee_id_bad_reserved2);
        println!("{res:?} `");
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
        let mut validator = Rdh0Validator::new();
        let rdh0 = Rdh0 {
            header_id: 7,
            header_size: 0x40,
            fee_id: FeeId(0x502A),
            priority_bit: 0,
            system_id: ITS_SYSTEM_ID,
            reserved0: 0,
        };
        let rdh0_2 = Rdh0 {
            header_id: 7,
            header_size: 0x40,
            fee_id: FeeId(0x502A),
            priority_bit: 0,
            system_id: ITS_SYSTEM_ID,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh0);
        assert!(res.is_ok());
        let res = validator.sanity_check(&rdh0_2);
        assert!(res.is_ok());
    }
    #[test]
    fn invalidate_rdh0_bad_header_id() {
        let mut validator = Rdh0Validator::default();
        let mut rdh0 = Rdh0 {
            header_id: 0x7,
            header_size: 0x40,
            fee_id: FeeId(0x502A),
            priority_bit: 0,
            system_id: ITS_SYSTEM_ID,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh0);
        assert!(res.is_ok());
        rdh0.header_id = 0x8; // Change to different header_id
        assert!(validator.sanity_check(&rdh0).is_err());
    }
    #[test]
    fn invalidate_rdh0_bad_header_size() {
        let mut validator = Rdh0Validator::new();
        let rdh0 = Rdh0 {
            header_id: 7,
            header_size: 0x3,
            fee_id: FeeId(0x502A),
            priority_bit: 0,
            system_id: ITS_SYSTEM_ID,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh0);
        println!("{res:?}");
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh0_bad_fee_id() {
        let mut validator = Rdh0Validator::default();
        let fee_id_bad_stave_number_is_48 = FeeId(0x30);
        let rdh0 = Rdh0 {
            header_id: 7,
            header_size: 0x40,
            fee_id: fee_id_bad_stave_number_is_48,
            priority_bit: 0,
            system_id: ITS_SYSTEM_ID,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh0);
        println!("{res:?}");
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh0_bad_system_id() {
        let mut validator = Rdh0Validator::new();
        let rdh0 = Rdh0 {
            header_id: 7,
            header_size: 0x40,
            fee_id: FeeId(0x502A),
            priority_bit: 0,
            system_id: 0x3,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh0);
        println!("{res:?}");
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh0_bad_reserved0() {
        let mut validator = Rdh0Validator::new();
        let rdh0 = Rdh0 {
            header_id: 7,
            header_size: 0x40,
            fee_id: FeeId(0x502A),
            priority_bit: 0,
            system_id: ITS_SYSTEM_ID,
            reserved0: 0x3,
        };
        let res = validator.sanity_check(&rdh0);
        println!("{res:?}");
        assert!(res.is_err());
    }

    // RDH1 sanity check
    #[test]
    fn validate_rdh1() {
        let validator = RDH1_VALIDATOR;
        let rdh1 = Rdh1::test_new(0, 0, 0);
        let res = validator.sanity_check(&rdh1);
        assert!(res.is_ok());
    }
    #[test]
    fn invalidate_rdh1_bad_reserved0() {
        let validator = RDH1_VALIDATOR;
        let rdh1 = Rdh1::test_new(0, 0, 1);
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
        let _reserved_bits_4_to_23_set: u32 = 0b1111_1111_1111_1111_1111_0000;
        let example_bad_detector_field = 0b1000_0000;
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
        let mut rdh_cru = CORRECT_RDH_CRU_V7;
        assert!(validator.sanity_check(&rdh_cru).is_ok());
        rdh_cru.rdh0.header_id = 0x0;
        let res = validator.sanity_check(&rdh_cru);
        println!("{res:?}");
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh_cru_v7_multiple_errors() {
        let mut validator = RdhCruSanityValidator::default();
        let mut rdh_cru = CORRECT_RDH_CRU_V7;
        rdh_cru.rdh0.header_size = 0x0;
        rdh_cru.rdh2.reserved0 = 0x1;
        rdh_cru.rdh3.detector_field = 0x5;
        rdh_cru.rdh3.reserved0 = 0x1;
        rdh_cru.reserved1 = 0x1;
        rdh_cru.reserved2 = 0x1;
        let fee_id_invalid_layer_is_7 = FeeId(0b0111_0000_0000_0000);
        rdh_cru.rdh0.fee_id = fee_id_invalid_layer_is_7;
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
        let mut rdh_cru = CORRECT_RDH_CRU_V6;
        let res = validator.sanity_check(&rdh_cru);
        println!("{res:?}");
        assert!(res.is_ok());
        rdh_cru.rdh0.header_id = 0x0;
        let res = validator.sanity_check(&rdh_cru);
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh_cru_v6_multiple_errors() {
        let mut validator = RdhCruSanityValidator::default();
        let mut rdh_cru = CORRECT_RDH_CRU_V6;
        rdh_cru.rdh0.header_size = 0x0;
        rdh_cru.rdh2.reserved0 = 0x1;
        rdh_cru.rdh3.detector_field = 0x5;
        rdh_cru.rdh3.reserved0 = 0x1;
        rdh_cru.reserved1 = 0x1;
        rdh_cru.reserved2 = 0x1;
        let fee_id_invalid_layer_is_7 = FeeId(0b0111_0000_0000_0000);
        rdh_cru.rdh0.fee_id = fee_id_invalid_layer_is_7;
        let res = validator.sanity_check(&rdh_cru);
        println!("{res:?}");
        assert!(res.is_err());
    }
}
