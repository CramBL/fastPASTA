use crate::data_words::rdh::{FeeId, Rdh0, Rdh1, Rdh2, Rdh3, RdhCRUv6, RdhCRUv7};
use std::fmt;
use std::fmt::Write as _;

// TODO: implement std:error::Error for all errors (or not? It's not program errors)
#[derive(Debug)]
pub enum GbtError {
    InvalidWord(String),
}
impl std::fmt::Display for GbtError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct FeeIdSanityValidator {
    pub reserved0: u8,
    layer_min_max: (u8, u8),
    pub reserved1: u8,
    fiber_uplink_min_max: (u8, u8),
    pub reserved2: u8,
    stave_number_min_max: (u8, u8),
}

impl FeeIdSanityValidator {
    const fn new(
        layer_min_max: (u8, u8),
        fiber_uplink_min_max: (u8, u8),
        stave_number_min_max: (u8, u8),
    ) -> Self {
        if layer_min_max.0 > layer_min_max.1 {
            panic!("Layer min must be smaller than layer max");
        }
        if fiber_uplink_min_max.0 > fiber_uplink_min_max.1 {
            panic!("Fiber uplink min must be smaller than fiber uplink max");
        }
        if stave_number_min_max.0 > stave_number_min_max.1 {
            panic!("Stave number min must be smaller than stave number max");
        }
        Self {
            reserved0: 0,
            layer_min_max,
            reserved1: 0,
            fiber_uplink_min_max,
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
        let stave_number_mask: u16 = 0b11_1111;
        let stave_number = (fee_id.0 & stave_number_mask) as u8;
        if stave_number < self.stave_number_min_max.0 || stave_number > self.stave_number_min_max.1
        {
            err_cnt += 1;
            write!(err_str, "{} = {} ", stringify!(stave_number), stave_number).unwrap();
        }

        // All values of fiber_uplink are valid in a sanity check
        // Extract fiber_uplink from 2 bits [9:8]
        // let fiber_uplink_mask: u16 = 0b11;
        // let fiber_uplink_lsb_idx: u8 = 8;
        // let fiber_uplink = ((fee_id.0 >> fiber_uplink_lsb_idx) & fiber_uplink_mask) as u8;
        // if fiber_uplink < self.fiber_uplink_min_max.0 || fiber_uplink > self.fiber_uplink_min_max.1
        // {
        //     todo!("Finish sanity check");
        // }
        // Extract layer from 3 bits [14:12]
        let layer_mask: u16 = 0b0111;
        let layer_lsb_idx: u8 = 12;
        let layer = ((fee_id.0 >> layer_lsb_idx) & layer_mask) as u8;

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

const FEE_ID_SANITY_VALIDATOR: FeeIdSanityValidator =
    FeeIdSanityValidator::new((0, 6), (0, 3), (0, 47));

pub struct Rdh0Validator {
    pub header_id: u8,
    pub header_size: u8,
    pub fee_id: FeeIdSanityValidator,
    pub priority_bit: u8,
    pub system_id: u8,
    pub reserved0: u16,
}

impl Rdh0Validator {
    pub fn sanity_check(&self, rdh0: &Rdh0) -> Result<(), String> {
        let mut err_str = String::new();
        let mut err_cnt: u8 = 0;
        if rdh0.header_id != self.header_id {
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
const ITS_SYSTEM_ID: u8 = 32;
pub const RDH0_V7_VALIDATOR: Rdh0Validator = Rdh0Validator {
    header_id: 7,
    header_size: 0x40,
    fee_id: FEE_ID_SANITY_VALIDATOR,
    priority_bit: 0,
    system_id: ITS_SYSTEM_ID,
    reserved0: 0,
};
pub const RDH0_V6_VALIDATOR: Rdh0Validator = Rdh0Validator {
    header_id: 6,
    header_size: 0x40,
    fee_id: FEE_ID_SANITY_VALIDATOR,
    priority_bit: 0,
    system_id: ITS_SYSTEM_ID,
    reserved0: 0,
};

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
        // Any orbit or bc are valid in a sanity check
        // if rdh1.bc() != self.valid_rdh1.bc() {
        //     err_cnt += 1;
        //     write!(err_str, "{} = {:#x} ", stringify!(bc), rdh1.bc()).unwrap();
        // }

        // if rdh1.orbit != self.valid_rdh1.orbit {
        //     err_cnt += 1;
        //     let tmp = rdh1.orbit;
        //     write!(err_str, "{} = {:#x} ", stringify!(orbit), tmp).unwrap();
        // }
        if err_cnt != 0 {
            return Err(err_str.to_owned());
        }
        Ok(())
    }
}
pub const RDH1_VALIDATOR: Rdh1Validator = Rdh1Validator {
    valid_rdh1: Rdh1::test_new(0, 0, 0),
};

pub struct Rdh2Validator {}
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
        // Any page counter is valid in a sanity check

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

pub struct Rdh3Validator {}

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

pub struct RdhCruv7Validator {
    rdh0_validator: &'static Rdh0Validator,
    rdh1_validator: &'static Rdh1Validator,
    rdh2_validator: &'static Rdh2Validator,
    rdh3_validator: &'static Rdh3Validator,
    //valid_dataformat_reserved0: DataformatReserved,
    // valid link IDs are 0-11 and 15
    // datawrapper ID is 0 or 1
}

impl RdhCruv7Validator {
    pub fn sanity_check(&self, rdh: &RdhCRUv7) -> Result<(), GbtError> {
        let mut err_str = String::from("RDH v7 sanity check failed: ");
        let mut err_cnt: u8 = 0;
        let mut rdh_errors: Vec<String> = vec![];
        match self.rdh0_validator.sanity_check(&rdh.rdh0) {
            Ok(_) => (),
            Err(e) => {
                err_cnt += 1;
                rdh_errors.push(e);
            }
        };
        match self.rdh1_validator.sanity_check(&rdh.rdh1) {
            Ok(_) => (),
            Err(e) => {
                err_cnt += 1;
                rdh_errors.push(e);
            }
        };
        match self.rdh2_validator.sanity_check(&rdh.rdh2) {
            Ok(_) => (),
            Err(e) => {
                err_cnt += 1;
                rdh_errors.push(e);
            }
        };
        match self.rdh3_validator.sanity_check(&rdh.rdh3) {
            Ok(_) => (),
            Err(e) => {
                err_cnt += 1;
                rdh_errors.push(e);
            }
        };

        // TODO: find out what the valid values for the cru id are
        if rdh.cru_id() > 0x1F {
            err_cnt += 1;
            let tmp = rdh.cru_id();
            write!(err_str, "{} = {:#x} ", stringify!(cru_id), tmp).unwrap();
        }
        if rdh.dw() > 1 {
            err_cnt += 1;
            let tmp = rdh.dw();
            write!(err_str, "{} = {:#x} ", stringify!(dw), tmp).unwrap();
        }
        if rdh.data_format() != 2 {
            err_cnt += 1;
            let tmp = rdh.data_format();
            write!(err_str, "{} = {:#x} ", stringify!(data_format), tmp).unwrap();
        }
        if rdh.reserved0() != 0 {
            err_cnt += 1;
            let tmp = rdh.reserved0();
            write!(err_str, "{} = {:#x} ", stringify!(reserved0), tmp).unwrap();
        }
        if rdh.reserved1 != 0 {
            err_cnt += 1;
            let tmp = rdh.reserved1;
            write!(err_str, "{} = {:#x} ", stringify!(reserved1), tmp).unwrap();
        }
        if rdh.reserved2 != 0 {
            err_cnt += 1;
            let tmp = rdh.reserved2;
            write!(err_str, "{} = {:#x} ", stringify!(reserved2), tmp).unwrap();
        }

        rdh_errors.into_iter().for_each(|e| {
            err_str.push_str(&e);
        });

        if err_cnt != 0 {
            return Err(GbtError::InvalidWord(err_str.to_owned()));
        }

        Ok(())
    }
}

pub const RDH_CRU_V7_VALIDATOR: RdhCruv7Validator = RdhCruv7Validator {
    rdh0_validator: &RDH0_V7_VALIDATOR,
    rdh1_validator: &RDH1_VALIDATOR,
    rdh2_validator: &RDH2_VALIDATOR,
    rdh3_validator: &RDH3_VALIDATOR,
};

pub struct RdhCruv6Validator {
    rdh0_validator: &'static Rdh0Validator,
    rdh1_validator: &'static Rdh1Validator,
    rdh2_validator: &'static Rdh2Validator,
    rdh3_validator: &'static Rdh3Validator,
    //valid_dataformat_reserved0: DataformatReserved,
    // valid link IDs are 0-11 and 15
    // datawrapper ID is 0 or 1
}

impl RdhCruv6Validator {
    pub fn sanity_check(&self, rdh: &RdhCRUv6) -> Result<(), GbtError> {
        let mut err_str = String::from("RDH v7 sanity check failed: ");
        let mut err_cnt: u8 = 0;
        let mut rdh_errors: Vec<String> = vec![];
        match self.rdh0_validator.sanity_check(&rdh.rdh0) {
            Ok(_) => (),
            Err(e) => {
                err_cnt += 1;
                rdh_errors.push(e);
            }
        };
        match self.rdh1_validator.sanity_check(&rdh.rdh1) {
            Ok(_) => (),
            Err(e) => {
                err_cnt += 1;
                rdh_errors.push(e);
            }
        };
        match self.rdh2_validator.sanity_check(&rdh.rdh2) {
            Ok(_) => (),
            Err(e) => {
                err_cnt += 1;
                rdh_errors.push(e);
            }
        };
        match self.rdh3_validator.sanity_check(&rdh.rdh3) {
            Ok(_) => (),
            Err(e) => {
                err_cnt += 1;
                rdh_errors.push(e);
            }
        };

        // TODO: find out what the valid values for the cru id are
        if rdh.cru_id() > 0x1F {
            err_cnt += 1;
            let tmp = rdh.cru_id();
            write!(err_str, "{} = {:#x} ", stringify!(cru_id), tmp).unwrap();
        }
        if rdh.dw() > 1 {
            err_cnt += 1;
            let tmp = rdh.dw();
            write!(err_str, "{} = {:#x} ", stringify!(dw), tmp).unwrap();
        }
        if rdh.reserved0 != 0 {
            err_cnt += 1;
            let tmp = rdh.reserved0;
            write!(err_str, "{} = {:#x} ", stringify!(reserved0), tmp).unwrap();
        }
        if rdh.reserved1 != 0 {
            err_cnt += 1;
            let tmp = rdh.reserved1;
            write!(err_str, "{} = {:#x} ", stringify!(reserved1), tmp).unwrap();
        }
        if rdh.reserved2 != 0 {
            err_cnt += 1;
            let tmp = rdh.reserved2;
            write!(err_str, "{} = {:#x} ", stringify!(reserved2), tmp).unwrap();
        }

        rdh_errors.into_iter().for_each(|e| {
            err_str.push_str(&e);
        });

        if err_cnt != 0 {
            return Err(GbtError::InvalidWord(err_str.to_owned()));
        }

        Ok(())
    }
}
pub const RDH_CRU_V6_VALIDATOR: RdhCruv6Validator = RdhCruv6Validator {
    rdh0_validator: &RDH0_V6_VALIDATOR,
    rdh1_validator: &RDH1_VALIDATOR,
    rdh2_validator: &RDH2_VALIDATOR,
    rdh3_validator: &RDH3_VALIDATOR,
};

pub struct RdhCruv7RunningChecker {
    pub expect_pages_counter: u16,
    pub last_rdh2: Option<Rdh2>,
}
impl RdhCruv7RunningChecker {
    pub fn new() -> Self {
        Self {
            expect_pages_counter: 0,
            last_rdh2: None,
        }
    }
    pub fn check(&mut self, rdh: &RdhCRUv7) -> Result<(), GbtError> {
        let mut err_str = String::from("RDH v7 running check failed: ");
        let mut err_cnt: u8 = 0;
        match rdh.rdh2.stop_bit {
            0 => {
                if rdh.rdh2.pages_counter != self.expect_pages_counter {
                    err_cnt += 1;
                    let tmp = rdh.rdh2.pages_counter;
                    write!(err_str, "{} = {:#x} ", stringify!(pages_counter), tmp).unwrap();
                }
                self.expect_pages_counter += 1;
            }
            1 => {
                if rdh.rdh2.pages_counter != self.expect_pages_counter {
                    err_cnt += 1;
                    let tmp = rdh.rdh2.pages_counter;
                    write!(err_str, "{} = {:#x} ", stringify!(pages_counter), tmp).unwrap();
                }
                self.expect_pages_counter = 0;
            }
            _ => {
                err_cnt += 1;
                write!(
                    err_str,
                    "{} = {:#x} ",
                    stringify!(stop_bit),
                    rdh.rdh2.stop_bit
                )
                .unwrap();
            }
        };

        if err_cnt != 0 {
            return Err(GbtError::InvalidWord(err_str.to_owned()));
        }

        self.last_rdh2 = Some(rdh.rdh2);

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use crate::data_words::rdh::{BcReserved, CruidDw, DataformatReserved};

    use super::*;

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
        println!("{:?} ", res);
        let res = validator.sanity_check(fee_id_bad_reserved1);
        println!("{:?} ", res);
        let res = validator.sanity_check(fee_id_bad_reserved2);
        println!("{:?} `", res);
        assert!(validator.sanity_check(fee_id_bad_reserved0).is_err());
        assert!(validator.sanity_check(fee_id_bad_reserved1).is_err());
        assert!(validator.sanity_check(fee_id_bad_reserved2).is_err());
    }
    #[test]
    fn invalidate_fee_id_bad_layer() {
        let validator = FEE_ID_SANITY_VALIDATOR;
        let fee_id_invalid_layer_is_7 = FeeId(0b0111_0000_0000_0000);
        let res = validator.sanity_check(fee_id_invalid_layer_is_7);
        println!("{:?}", res);
        assert!(validator.sanity_check(fee_id_invalid_layer_is_7).is_err());
    }

    // All values of fiber_uplink are valid in a sanity check
    // #[test]
    // fn invalidate_fee_id_bad_fiber_uplink() {
    //     let validator = FEE_ID_SANITY_VALIDATOR;
    //     let invalid_value = ??
    //     let fee_id_invalid_fiber_uplink_is_x = FeeId(invalid_value);
    //     assert!(validator.sanity_check(fee_id_invalid_fiber_uplink_is_x).is_err());
    // }

    #[test]
    fn invalidate_fee_id_bad_stave_number() {
        let validator = FEE_ID_SANITY_VALIDATOR;
        let fee_id_bad_stave_number_is_48 = FeeId(0x30);
        let res = validator.sanity_check(fee_id_bad_stave_number_is_48);
        println!("{:?}", res);
        assert!(res.is_err());
    }
    // RDH0 sanity check
    #[test]
    fn validate_rdh0() {
        let validator = RDH0_V7_VALIDATOR;
        let rdh0 = Rdh0 {
            header_id: 7,
            header_size: 0x40,
            fee_id: FeeId(0x502A),
            priority_bit: 0,
            system_id: ITS_SYSTEM_ID,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh0);
        assert!(res.is_ok());
    }
    #[test]
    fn invalidate_rdh0_bad_header_id() {
        let validator = RDH0_V7_VALIDATOR;
        let rdh0 = Rdh0 {
            header_id: 0x3,
            header_size: 0x40,
            fee_id: FeeId(0x502A),
            priority_bit: 0,
            system_id: ITS_SYSTEM_ID,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh0);
        println!("{:?}", res);
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh0_bad_header_size() {
        let validator = RDH0_V7_VALIDATOR;
        let rdh0 = Rdh0 {
            header_id: 7,
            header_size: 0x3,
            fee_id: FeeId(0x502A),
            priority_bit: 0,
            system_id: ITS_SYSTEM_ID,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh0);
        println!("{:?}", res);
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh0_bad_fee_id() {
        let validator = RDH0_V7_VALIDATOR;
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
        println!("{:?}", res);
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh0_bad_system_id() {
        let validator = RDH0_V7_VALIDATOR;
        let rdh0 = Rdh0 {
            header_id: 7,
            header_size: 0x40,
            fee_id: FeeId(0x502A),
            priority_bit: 0,
            system_id: 0x3,
            reserved0: 0,
        };
        let res = validator.sanity_check(&rdh0);
        println!("{:?}", res);
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh0_bad_reserved0() {
        let validator = RDH0_V7_VALIDATOR;
        let rdh0 = Rdh0 {
            header_id: 7,
            header_size: 0x40,
            fee_id: FeeId(0x502A),
            priority_bit: 0,
            system_id: ITS_SYSTEM_ID,
            reserved0: 0x3,
        };
        let res = validator.sanity_check(&rdh0);
        println!("{:?}", res);
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
        println!("{:?}", res);
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
        println!("{:?}", res);
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
        println!("{:?}", res);
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
        println!("{:?}", res);
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
        println!("{:?}", res);
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
        println!("{:?}", res);
        assert!(res.is_err());
    }

    // RDH-CRU v7 sanity check
    // Data for use in tests:
    const CORRECT_RDH_CRU: RdhCRUv7 = RdhCRUv7 {
        rdh0: Rdh0 {
            header_id: 0x7,
            header_size: 0x40,
            fee_id: FeeId(0x502A),
            priority_bit: 0x0,
            system_id: 0x20,
            reserved0: 0,
        },
        offset_new_packet: 0x13E0,
        memory_size: 0x13E0,
        link_id: 0x0,
        packet_counter: 0x0,
        cruid_dw: CruidDw(0x0018),
        rdh1: Rdh1 {
            bc_reserved0: BcReserved(0x0),
            orbit: 0x0b7dd575,
        },
        dataformat_reserved0: DataformatReserved(0x2),
        rdh2: Rdh2 {
            trigger_type: 0x00006a03,
            pages_counter: 0x0,
            stop_bit: 0x0,
            reserved0: 0x0,
        },
        reserved1: 0x0,
        rdh3: Rdh3 {
            detector_field: 0x0,
            par_bit: 0x0,
            reserved0: 0x0,
        },
        reserved2: 0x0,
    };

    #[test]
    fn validate_rdh_cru_v7() {
        let validator = RDH_CRU_V7_VALIDATOR;
        let res = validator.sanity_check(&CORRECT_RDH_CRU);
        assert!(res.is_ok());
    }
    #[test]
    fn invalidate_rdh_cru_v7_bad_header_id() {
        let validator = RDH_CRU_V7_VALIDATOR;
        let mut rdh_cru = CORRECT_RDH_CRU;
        rdh_cru.rdh0.header_id = 0x0;
        let res = validator.sanity_check(&rdh_cru);
        println!("{:?}", res);
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh_cru_v7_multiple_errors() {
        let validator = RDH_CRU_V7_VALIDATOR;
        let mut rdh_cru = CORRECT_RDH_CRU;
        rdh_cru.rdh0.header_size = 0x0;
        rdh_cru.rdh2.reserved0 = 0x1;
        rdh_cru.rdh3.detector_field = 0x5;
        rdh_cru.rdh3.reserved0 = 0x1;
        rdh_cru.reserved1 = 0x1;
        rdh_cru.reserved2 = 0x1;
        let fee_id_invalid_layer_is_7 = FeeId(0b0111_0000_0000_0000);
        rdh_cru.rdh0.fee_id = fee_id_invalid_layer_is_7;
        let res = validator.sanity_check(&rdh_cru);
        println!("{:?}", res);
        assert!(res.is_err());
    }

    // RDH-CRU v6 sanity check
    // Data for use in tests:
    const CORRECT_RDH_CRU_V6: RdhCRUv6 = RdhCRUv6 {
        rdh0: Rdh0 {
            header_id: 0x6,
            header_size: 0x40,
            fee_id: FeeId(0x502A),
            priority_bit: 0x0,
            system_id: 0x20,
            reserved0: 0,
        },
        offset_new_packet: 0x13E0,
        memory_size: 0x13E0,
        link_id: 0x2,
        packet_counter: 0x1,
        cruid_dw: CruidDw(0x0018),
        rdh1: Rdh1 {
            bc_reserved0: BcReserved(0x0),
            orbit: 0x0b7dd575,
        },
        reserved0: 0x0,
        rdh2: Rdh2 {
            trigger_type: 0x00006a03,
            pages_counter: 0x0,
            stop_bit: 0x0,
            reserved0: 0x0,
        },
        reserved1: 0x0,
        rdh3: Rdh3 {
            detector_field: 0x0,
            par_bit: 0x0,
            reserved0: 0x0,
        },
        reserved2: 0x0,
    };

    #[test]
    fn validate_rdh_cru_v6() {
        let validator = RDH_CRU_V6_VALIDATOR;
        let res = validator.sanity_check(&CORRECT_RDH_CRU_V6);
        assert!(res.is_ok());
    }
    #[test]
    fn invalidate_rdh_cru_v6_bad_header_id() {
        let validator = RDH_CRU_V6_VALIDATOR;
        let mut rdh_cru = CORRECT_RDH_CRU_V6;
        rdh_cru.rdh0.header_id = 0x0;
        let res = validator.sanity_check(&rdh_cru);
        println!("{:?}", res);
        assert!(res.is_err());
    }
    #[test]
    fn invalidate_rdh_cru_v6_multiple_errors() {
        let validator = RDH_CRU_V6_VALIDATOR;
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
        println!("{:?}", res);
        assert!(res.is_err());
    }
}
