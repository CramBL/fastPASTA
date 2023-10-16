//! Contains the struct definition of the TDH
//!
use std::fmt::Display;

use byteorder::{ByteOrder, LittleEndian};

use super::{display_byte_slice, StatusWord};

/// Struct to represent the TDH status word
#[repr(packed)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Tdh {
    // 11:0 trigger_type
    // 12: internal_trigger, 13: no_data, 14: continuation, 15: reserved
    trigger_type_internal_trigger_no_data_continuation_reserved2: u16,
    trigger_bc_reserved1: u16,     // 27:16 trigger_bc, 31:28 reserved,
    pub(crate) trigger_orbit: u32, // 63:32
    // ID 0xe8
    reserved0_id: u16, // 71:64 reserved, 79:72 id
}
impl Tdh {
    /// Maximum value of the trigger_bc field
    pub const MAX_BC: u16 = 3563;
    /// Returns the integer value of the reserved0 field
    pub fn reserved0(&self) -> u16 {
        self.reserved0_id & 0xFF
    }

    /// Returns the integer value of the reserved1 field
    pub fn reserved1(&self) -> u16 {
        self.trigger_bc_reserved1 & 0xF000 // doesn't need shift as it should just be checked if equal to 0
    }

    /// Returns the integer value of the trigger_bc field
    pub fn trigger_bc(&self) -> u16 {
        self.trigger_bc_reserved1 & 0x0FFF
    }

    /// Returns the integer value of the reserved2 field
    pub fn reserved2(&self) -> u16 {
        // 15th bit is reserved
        self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b1000_0000_0000_0000
    }

    /// Returns the integer value of the continuation field
    pub fn continuation(&self) -> u16 {
        // 14th bit is continuation
        (self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b100_0000_0000_0000)
            >> 14
    }

    /// Returns the integer value of the no_data field
    pub fn no_data(&self) -> u16 {
        // 13th bit is no_data
        (self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b10_0000_0000_0000)
            >> 13
    }

    /// Returns the integer value of the internal_trigger field
    pub fn internal_trigger(&self) -> u16 {
        // 12th bit is internal_trigger
        (self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b1_0000_0000_0000)
            >> 12
    }

    /// Returns the integer value of the trigger_type field
    ///
    /// Beware! Only 12 LSB are valid!
    pub fn trigger_type(&self) -> u16 {
        // 11:0 is trigger_type
        self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b1111_1111_1111
    }
}

impl Display for Tdh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_byte_slice(self, f)
    }
}

impl StatusWord for Tdh {
    fn id(&self) -> u8 {
        (self.reserved0_id >> 8) as u8
    }

    fn is_reserved_0(&self) -> bool {
        self.reserved0() == 0 && self.reserved1() == 0 && self.reserved2() == 0
    }

    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(Tdh {
            trigger_type_internal_trigger_no_data_continuation_reserved2: LittleEndian::read_u16(
                &buf[0..=1],
            ),
            trigger_bc_reserved1: LittleEndian::read_u16(&buf[2..=3]),
            trigger_orbit: LittleEndian::read_u32(&buf[4..=7]),
            reserved0_id: LittleEndian::read_u16(&buf[8..=9]),
        })
    }
}

#[cfg(test)]
mod tests {
    use alice_protocol_reader::prelude::ByteSlice;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn tdh_read_write() {
        const VALID_ID: u8 = 0xE8;
        let raw_data_tdh = [0x03, 0x1A, 0x00, 0x00, 0x75, 0xD5, 0x7D, 0x0B, 0x00, 0xE8];
        const TRIGGER_TYPE: u16 = 0xA03;
        const INTERNAL_TRIGGER: u16 = 1; // 0x1
        const NO_DATA: u16 = 0; // 0x0
        const CONTINUATION: u16 = 0; // 0x0
        const TRIGGER_BC: u16 = 0;
        const TRIGGER_ORBIT: u32 = 0x0B7DD575;
        let tdh = Tdh::load(&mut raw_data_tdh.as_slice()).unwrap();
        println!("{tdh}");
        assert_eq!(tdh.id(), VALID_ID);
        assert!(tdh.is_reserved_0());
        assert_eq!(tdh.trigger_type(), TRIGGER_TYPE);
        assert_eq!(tdh.internal_trigger(), INTERNAL_TRIGGER);
        assert_eq!(tdh.no_data(), NO_DATA);
        assert_eq!(tdh.continuation(), CONTINUATION);
        assert_eq!(tdh.trigger_bc(), TRIGGER_BC);
        let trigger_orbit = tdh.trigger_orbit;
        assert_eq!(trigger_orbit, TRIGGER_ORBIT);
        let loaded_tdh = Tdh::load(&mut tdh.to_byte_slice()).unwrap();
        assert_eq!(tdh, loaded_tdh);
    }
}
