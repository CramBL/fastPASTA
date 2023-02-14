use crate::data_words::rdh::{BcReserved, FeeId, Rdh0, Rdh1};
use std::fmt::Write as _;
use std::fmt::{self, write};
use std::io::Write as _;

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
    pub fn sanity_check(&self, rdh0: &Rdh0) -> Result<(), GbtError> {
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
            write!(err_str, "{} = {:#x} ", stringify!(reserved0), tmp).unwrap();
        }
        if err_cnt != 0 {
            return Err(GbtError::InvalidWord(err_str.to_owned()));
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

pub struct Rdh1Validator {
    valid_rdh1: Rdh1,
}
impl Rdh1Validator {
    pub fn sanity_check(&self, rdh1: &Rdh1) -> Result<(), GbtError> {
        let mut err_str = String::new();
        let mut err_cnt: u8 = 0;
        if rdh1.reserved0() != self.valid_rdh1.reserved0() {
            err_cnt += 1;
            write!(
                err_str,
                "{} = {:#x} ",
                stringify!(reserved0),
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
            return Err(GbtError::InvalidWord(err_str.to_owned()));
        }
        Ok(())
    }
}
pub const RDH1_V7_VALIDATOR: Rdh1Validator = Rdh1Validator {
    valid_rdh1: Rdh1::test_new(0, 0, 0),
};

#[cfg(test)]
mod tests {
    use crate::macros::print;

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
        let validator = RDH1_V7_VALIDATOR;
        let rdh1 = Rdh1::test_new(0, 0, 0);
        let res = validator.sanity_check(&rdh1);
        assert!(res.is_ok());
    }
    #[test]
    fn invalidate_rdh1_bad_reserved0() {
        let validator = RDH1_V7_VALIDATOR;
        let rdh1 = Rdh1::test_new(0, 0, 1);
        let res = validator.sanity_check(&rdh1);
        println!("{:?}", res);
        assert!(res.is_err());
    }
}
