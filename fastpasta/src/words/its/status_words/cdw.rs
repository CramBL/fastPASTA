//! Contains the struct definition of the CDW

use super::{display_byte_slice, StatusWord};
use crate::util::*;

/// Struct representing the CDW.
#[repr(packed)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Cdw {
    calibration_word_index_lsb_calibration_user_fields: u64, // 63:48 calibration_word_index_LSB 47:0 calibration_user_fields
    calibration_word_index_msb: u8,                          // 71:64 calibration_word_index_MSB
    // ID: 0xF8
    id: u8,
}

impl Cdw {
    /// Returns the integer value of the calibration_word_index field.
    pub fn calibration_word_index(&self) -> u32 {
        ((self.calibration_word_index_msb as u32) << 16)
            | ((self.calibration_word_index_lsb_calibration_user_fields >> 48) as u32)
    }
    /// Returns the integer value of the calibration_user_fields field.
    pub fn calibration_user_fields(&self) -> u64 {
        self.calibration_word_index_lsb_calibration_user_fields & 0xffff_ffff_ffff
    }
}

impl fmt::Display for Cdw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_byte_slice(self, f)
    }
}

impl StatusWord for Cdw {
    fn id(&self) -> u8 {
        self.id
    }

    fn is_reserved_0(&self) -> bool {
        true // No reserved bits
    }

    fn from_buf(buf: &[u8]) -> Result<Self, io::Error> {
        Ok(Self {
            calibration_word_index_lsb_calibration_user_fields: LittleEndian::read_u64(&buf[0..=7]),
            calibration_word_index_msb: buf[8],
            id: buf[9],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alice_protocol_reader::prelude::ByteSlice;
    use pretty_assertions::assert_eq;

    #[test]
    fn cdw_read_write() {
        const VALID_ID: u8 = 0xF8;
        let raw_data_cdw = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0xF8];

        let cdw = Cdw::load(&mut raw_data_cdw.as_slice()).unwrap();

        assert_eq!(cdw.id(), VALID_ID);

        assert!(cdw.is_reserved_0());
        assert_eq!(cdw.calibration_user_fields(), 0x050403020100);
        assert_eq!(cdw.calibration_word_index(), 0x080706);
        let loaded_cdw = Cdw::load(&mut cdw.to_byte_slice()).unwrap();
        assert_eq!(cdw, loaded_cdw);
    }
}
