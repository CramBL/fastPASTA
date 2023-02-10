use crate::data_words::rdh::{FeeId, Rdh0};

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
    fn sanity_check(&self, fee_id: FeeId) -> Result<(), GbtError> {
        // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number
        // 5:0 stave number
        // 7:6 reserved
        // 9:8 fiber uplink
        // 11:10 reserved
        // 14:12 layer
        // 15 reserved
        // Extract mask over reserved bits and check if it is 0
        let reserved_bits_mask: u16 = 0b1000_1100_1100_0000;
        let reserved_bits = fee_id.0 & reserved_bits_mask;
        if reserved_bits != 0 {
            return Err(GbtError::InvalidWord);
        }
        // Extract stave_number from 6 LSB [5:0]
        let stave_number_mask: u16 = 0b11_1111;
        let stave_number = (fee_id.0 & stave_number_mask) as u8;
        if stave_number < self.stave_number_min_max.0 || stave_number > self.stave_number_min_max.1
        {
            return Err(GbtError::InvalidWord);
        }
        // All values of fiber_uplink are valid in a sanity check
        // Extract fiber_uplink from 2 bits [9:8]
        // let fiber_uplink_mask: u16 = 0b11;
        // let fiber_uplink_lsb_idx: u8 = 8;
        // let fiber_uplink = ((fee_id.0 >> fiber_uplink_lsb_idx) & fiber_uplink_mask) as u8;
        // if fiber_uplink < self.fiber_uplink_min_max.0 || fiber_uplink > self.fiber_uplink_min_max.1
        // {
        //     return Err(GbtError::InvalidWord);
        // }
        // Extract layer from 3 bits [14:12]
        let layer_mask: u16 = 0b0111;
        let layer_lsb_idx: u8 = 12;
        let layer = ((fee_id.0 >> layer_lsb_idx) & layer_mask) as u8;

        if layer < self.layer_min_max.0 || layer > self.layer_min_max.1 {
            return Err(GbtError::InvalidWord);
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
        if rdh0.header_id != self.header_id {
            return Err(GbtError::InvalidWord);
        }
        if rdh0.header_size != self.header_size {
            return Err(GbtError::InvalidWord);
        }
        if let Err(e) = self.fee_id.sanity_check(rdh0.fee_id) {
            return Err(e);
        }
        if rdh0.priority_bit != self.priority_bit {
            return Err(GbtError::InvalidWord);
        }
        if rdh0.system_id != self.system_id {
            return Err(GbtError::InvalidWord);
        }
        if rdh0.reserved0 != self.reserved0 {
            return Err(GbtError::InvalidWord);
        }
        Ok(())
    }
}
const ITS_SYSTEM_ID: u8 = 32;
pub const RDH0_VALIDATOR: Rdh0Validator = Rdh0Validator {
    header_id: 7,
    header_size: 0x40,
    fee_id: FEE_ID_SANITY_VALIDATOR,
    priority_bit: 0,
    system_id: ITS_SYSTEM_ID,
    reserved0: 0,
};

// TODO: implement std:error::Error for all errors
pub enum GbtError {
    InvalidWord,
}

#[cfg(test)]
mod tests {
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
        assert!(validator.sanity_check(fee_id_bad_reserved0).is_err());
        assert!(validator.sanity_check(fee_id_bad_reserved1).is_err());
        assert!(validator.sanity_check(fee_id_bad_reserved2).is_err());
    }
    #[test]
    fn invalidate_fee_id_bad_layer() {
        let validator = FEE_ID_SANITY_VALIDATOR;
        let fee_id_invalid_layer_is_7 = FeeId(0b0111_0000_0000_0000);
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
        assert!(validator
            .sanity_check(fee_id_bad_stave_number_is_48)
            .is_err());
    }
}
