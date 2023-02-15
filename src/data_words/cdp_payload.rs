use std::fmt::Debug;

use byteorder::{ByteOrder, LittleEndian};

use crate::{pretty_print_hex_field, pretty_print_name_hex_fields, ByteSlice, GbtWord};

#[repr(packed)]
#[derive(PartialEq)]
pub struct Ihw {
    // Total of 80 bits
    id: u16,           // 79:72
    reserved: u32,     // 71:28
    active_lanes: u32, // 27:0
}

impl Ihw {
    pub fn id(&self) -> u16 {
        self.id >> 8
    }
    pub fn reserved(&self) -> u64 {
        let four_lsb: u8 = ((self.active_lanes >> 28) & 0xF) as u8;
        let eight_msb = self.id & 0xFF;
        (eight_msb as u64) << 36 | (self.reserved as u64) << 4 | (four_lsb as u64)
    }
    pub fn active_lanes(&self) -> u32 {
        self.active_lanes & 0xFFFFFFF
    }
}

impl GbtWord for Ihw {
    fn print(&self) {
        pretty_print_name_hex_fields!(Ihw, self, id, reserved, active_lanes);
    }

    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        // Create a helper macro for loading an array of the given size from
        // the reader.
        macro_rules! load_bytes {
            ($size:literal) => {{
                // Create a buffer array of the given size
                let mut buf = [0u8; $size];
                // Read into the buffer
                reader.read_exact(&mut buf)?;
                buf
            }};
        }
        Ok(Ihw {
            id: LittleEndian::read_u16(&load_bytes!(2)),
            reserved: LittleEndian::read_u32(&load_bytes!(4)),
            active_lanes: LittleEndian::read_u32(&load_bytes!(4)),
        })
    }
}

impl ByteSlice for Ihw {
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { crate::any_as_u8_slice(self) }
    }
}

impl Debug for Ihw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.id();
        let reserved = self.reserved();
        let active_lanes = self.active_lanes();
        write!(f, "{:x} {:x} {:x}", id, reserved, active_lanes)
    }
}
