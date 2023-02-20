use std::fmt::Debug;

use byteorder::{ByteOrder, LittleEndian};

use crate::{pretty_print_hex_field, pretty_print_name_hex_fields, ByteSlice};

pub trait StatusWord: std::fmt::Debug + PartialEq + Sized + ByteSlice {
    fn id(&self) -> u16;
    fn print(&self);
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
}

#[repr(packed)]
pub struct Ihw {
    // Total of 80 bits
    id: u16,           // 79:72
    reserved: u32,     // 71:28
    active_lanes: u32, // 27:0
}

impl Ihw {
    pub fn reserved(&self) -> u64 {
        let four_lsb: u8 = ((self.active_lanes >> 28) & 0xF) as u8;
        let eight_msb = self.id & 0xFF;
        (eight_msb as u64) << 36 | (self.reserved as u64) << 4 | (four_lsb as u64)
    }
    pub fn active_lanes(&self) -> u32 {
        self.active_lanes & 0xFFFFFFF
    }
}

impl StatusWord for Ihw {
    fn id(&self) -> u16 {
        self.id >> 8
    }
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

impl PartialEq for Ihw {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.reserved == other.reserved
            && self.active_lanes == other.active_lanes
    }
}

#[repr(packed)]
pub struct Tdh {
    // ID 0xe8
    id_reserved0: u16,         // 79:72, 71:64
    pub trigger_orbit: u32,    // 63:32
    reserved1_trigger_bc: u16, // 31:28 reserved, 27:16 trigger_bc
    // 15: reserved, 14: continuation, 13: no_data, 12: internal_trigger, 11:0 trigger_type
    reserved2_continuation_no_data_internal_trigger_trigger_type: u16,
}
impl Tdh {
    fn reserved0(&self) -> u16 {
        self.id_reserved0 & 0xFF
    }

    fn reserved1(&self) -> u16 {
        self.reserved1_trigger_bc & 0xF000 // doesn't need shift as it should just be checked if equal to 0
    }

    fn trigger_bc(&self) -> u16 {
        self.reserved1_trigger_bc & 0x0FFF
    }

    fn reserved2(&self) -> u16 {
        // 15th bit is reserved
        self.reserved2_continuation_no_data_internal_trigger_trigger_type & 0b1000_0000_0000_0000
    }
    fn continuation(&self) -> u16 {
        // 14th bit is continuation
        self.reserved2_continuation_no_data_internal_trigger_trigger_type
            & 0b100_0000_0000_0000 >> 14
    }

    fn no_data(&self) -> u16 {
        // 13th bit is no_data
        self.reserved2_continuation_no_data_internal_trigger_trigger_type
            & 0b10_0000_0000_0000 >> 13
    }

    fn internal_trigger(&self) -> u16 {
        // 12th bit is internal_trigger
        self.reserved2_continuation_no_data_internal_trigger_trigger_type & 0b1_0000_0000_0000 >> 12
    }

    fn trigger_type(&self) -> u16 {
        // 11:0 is trigger_type
        self.reserved2_continuation_no_data_internal_trigger_trigger_type & 0b1111_1111_1111
    }
}

impl StatusWord for Tdh {
    fn id(&self) -> u16 {
        self.id_reserved0 >> 8
    }

    fn print(&self) {}

    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        macro_rules! load_bytes {
            ($size:literal) => {{
                // Create a buffer array of the given size
                let mut buf = [0u8; $size];
                // Read into the buffer
                reader.read_exact(&mut buf)?;
                buf
            }};
        }
        Ok(Tdh {
            id_reserved0: LittleEndian::read_u16(&load_bytes!(2)),
            trigger_orbit: LittleEndian::read_u32(&load_bytes!(4)),
            reserved1_trigger_bc: LittleEndian::read_u16(&load_bytes!(2)),
            reserved2_continuation_no_data_internal_trigger_trigger_type: LittleEndian::read_u16(
                &load_bytes!(2),
            ),
        })
    }
}

impl ByteSlice for Tdh {
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { crate::any_as_u8_slice(self) }
    }
}

impl Debug for Tdh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.id();
        let reserved0 = self.reserved0();
        let trigger_orbit = self.trigger_orbit;
        let reserved1 = self.reserved1();
        let trigger_bc = self.trigger_bc();
        let reserved2 = self.reserved2();
        let continuation = self.continuation();
        let no_data = self.no_data();
        let internal_trigger = self.internal_trigger();
        let trigger_type = self.trigger_type();
        write!(
            f,
            "{:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x}",
            id,
            reserved0,
            trigger_orbit,
            reserved1,
            trigger_bc,
            reserved2,
            continuation,
            no_data,
            internal_trigger,
            trigger_type
        )
    }
}

impl PartialEq for Tdh {
    fn eq(&self, other: &Self) -> bool {
        self.id_reserved0 == other.id_reserved0
            && self.trigger_orbit == other.trigger_orbit
            && self.reserved1_trigger_bc == other.reserved1_trigger_bc
            && self.reserved2_continuation_no_data_internal_trigger_trigger_type
                == other.reserved2_continuation_no_data_internal_trigger_trigger_type
    }
}
