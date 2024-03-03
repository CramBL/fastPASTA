//! Contains the struct definition of the IHW
use super::{display_byte_slice, StatusWord};
use crate::util::*;

/// Struct to represent the IHW status word
#[repr(packed)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ihw {
    // Total of 80 bits
    // ID: 0xE0
    active_lanes: u32, // 27:0
    reserved: u32,     // 71:28
    id: u16,           // 79:72
}

impl Ihw {
    /// Returns the integer value of the reserved bits
    pub fn reserved(&self) -> u64 {
        let four_lsb: u8 = ((self.active_lanes >> 28) & 0xF) as u8;
        let eight_msb = self.id & 0xFF;
        (eight_msb as u64) << 36 | (self.reserved as u64) << 4 | (four_lsb as u64)
    }
    /// Returns the integer value of the active lanes field
    pub fn active_lanes(&self) -> u32 {
        self.active_lanes & 0xFFFFFFF
    }
}

impl fmt::Display for Ihw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_byte_slice(self, f)
    }
}

impl StatusWord for Ihw {
    fn id(&self) -> u8 {
        (self.id >> 8) as u8
    }

    fn is_reserved_0(&self) -> bool {
        self.reserved() == 0
    }

    fn from_buf(buf: &[u8]) -> Result<Self, io::Error> {
        Ok(Ihw {
            active_lanes: LittleEndian::read_u32(&buf[0..=3]),
            reserved: LittleEndian::read_u32(&buf[4..=7]),
            id: LittleEndian::read_u16(&buf[8..=9]),
        })
    }
}

#[cfg(test)]
mod tests {
    use alice_protocol_reader::prelude::ByteSlice;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn ihw_read_write() {
        const VALID_ID: u8 = 0xE0;
        const ACTIVE_LANES_14_ACTIVE: u32 = 0x3F_FF;
        let raw_data_ihw = [0xFF, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0];
        let ihw = Ihw::load(&mut raw_data_ihw.as_slice()).unwrap();
        assert_eq!(ihw.id(), VALID_ID);
        assert!(ihw.is_reserved_0());
        assert_eq!(ihw.active_lanes(), ACTIVE_LANES_14_ACTIVE);
        println!("{ihw:#?}");
        let loaded_ihw = Ihw::load(&mut ihw.to_byte_slice()).unwrap();
        println!("{loaded_ihw:?}");
        assert_eq!(ihw, loaded_ihw);
    }
}
