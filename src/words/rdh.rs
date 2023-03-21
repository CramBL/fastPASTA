// ITS data format: https://gitlab.cern.ch/alice-its-wp10-firmware/RU_mainFPGA/-/wikis/ITS%20Data%20Format#Introduction
use super::lib::RdhSubWord;
use byteorder::{ByteOrder, LittleEndian};
use std::fmt::{self, Debug, Display};

// Newtype pattern used to enforce type safety on fields that are not byte-aligned
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(packed)]
pub(crate) struct CruidDw(pub(crate) u16); // 12 bit cru_id, 4 bit dw
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(packed)]
pub(crate) struct BcReserved(pub(crate) u32); // 12 bit bc, 20 bit reserved
#[repr(packed)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct DataformatReserved(pub(crate) u64); // 8 bit data_format, 56 bit reserved0
#[repr(packed)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct FeeId(pub(crate) u16); // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number
                                         // Exaxmple: L4_12 -> Layer 4 stave 12 = 0b0100_00XX_0000_1100

#[repr(packed)]
pub struct Rdh0 {
    // Represents 64 bit
    pub header_id: u8,
    pub header_size: u8,
    pub(crate) fee_id: FeeId, // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number
    pub priority_bit: u8,
    pub system_id: u8,
    pub reserved0: u16,
}

impl Display for Rdh0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_fee = self.fee_id.0;
        write!(
            f,
            "{:<6}{:<7}{:<7}{:<6}",
            self.header_id, self.header_size, tmp_fee, self.system_id
        )
    }
}

impl RdhSubWord for Rdh0 {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Rdh0, std::io::Error> {
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
        Ok(Rdh0 {
            header_id: load_bytes!(1)[0],
            header_size: load_bytes!(1)[0],
            fee_id: FeeId(LittleEndian::read_u16(&load_bytes!(2))),
            priority_bit: load_bytes!(1)[0],
            system_id: load_bytes!(1)[0],
            reserved0: LittleEndian::read_u16(&load_bytes!(2)),
        })
    }
}
impl Debug for Rdh0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_header_id = self.header_id;
        let tmp_header_size = self.header_size;
        let tmp_fee_id = self.fee_id;
        let tmp_priority_bit = self.priority_bit;
        let tmp_system_id = self.system_id;
        let tmp_reserved0 = self.reserved0;

        write!(f, "Rdh0: header_id: {tmp_header_id:x?}, header_size: {tmp_header_size:x?}, fee_id: {tmp_fee_id:x?}, priority_bit: {tmp_priority_bit:x?}, system_id: {tmp_system_id:x?}, reserved0: {tmp_reserved0:x?}")
    }
}
impl PartialEq for Rdh0 {
    fn eq(&self, other: &Self) -> bool {
        self.header_id == other.header_id
            && self.header_size == other.header_size
            && self.fee_id == other.fee_id
            && self.priority_bit == other.priority_bit
            && self.system_id == other.system_id
            && self.reserved0 == other.reserved0
    }
}

#[repr(packed)]
pub struct Rdh1 {
    // Rdh1 is 64 bit total
    pub(crate) bc_reserved0: BcReserved, //bunch counter 12 bit + reserved 20 bit
    pub orbit: u32,                      // 32 bit
}

impl Rdh1 {
    // only meant for unit tests
    pub const fn test_new(bc: u16, orbit: u32, reserved0: u32) -> Self {
        Rdh1 {
            bc_reserved0: BcReserved((bc as u32) | (reserved0 << 12)),
            orbit,
        }
    }

    pub fn bc(&self) -> u16 {
        (self.bc_reserved0.0 & 0x0FFF) as u16
    }
    pub fn reserved0(&self) -> u32 {
        self.bc_reserved0.0 >> 12
    }
}

impl RdhSubWord for Rdh1 {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Rdh1, std::io::Error> {
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

        Ok(Rdh1 {
            bc_reserved0: BcReserved(LittleEndian::read_u32(&load_bytes!(4))),
            orbit: LittleEndian::read_u32(&load_bytes!(4)),
        })
    }
}
impl Debug for Rdh1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_bc = self.bc();
        let tmp_reserved0 = self.reserved0();
        let tmp_orbit = self.orbit;
        write!(
            f,
            "Rdh1: bc: {tmp_bc:x?}, reserved0: {tmp_reserved0:x?}, orbit: {tmp_orbit:x?}"
        )
    }
}
impl PartialEq for Rdh1 {
    fn eq(&self, other: &Self) -> bool {
        self.bc_reserved0 == other.bc_reserved0 && self.orbit == other.orbit
    }
}

impl Display for Rdh1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_orbit = self.orbit;
        let orbit_as_hex = format!("{tmp_orbit:#x}");
        write!(f, "{:<5}{:<12}", self.bc(), orbit_as_hex)
    }
}

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct Rdh2 {
    pub trigger_type: u32, // 32 bit
    pub pages_counter: u16,
    pub stop_bit: u8,
    pub reserved0: u8,
}

impl Rdh2 {
    // Checks if the 4th bit of the trigger type is set
    #[inline]
    pub fn is_pht_trigger(&self) -> bool {
        self.trigger_type >> 4 & 0x1 == 1
    }
}

impl RdhSubWord for Rdh2 {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Rdh2, std::io::Error> {
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

        Ok(Rdh2 {
            trigger_type: LittleEndian::read_u32(&load_bytes!(4)),
            pages_counter: LittleEndian::read_u16(&load_bytes!(2)),
            stop_bit: load_bytes!(1)[0],
            reserved0: load_bytes!(1)[0],
        })
    }
}
impl Debug for Rdh2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_trigger_type = self.trigger_type;
        let tmp_pages_counter = self.pages_counter;
        let tmp_stop_bit = self.stop_bit;
        write!(
            f,
            "Rdh2: trigger_type: {tmp_trigger_type:X?}, pages_counter: {tmp_pages_counter:X?}, stop_bit: {tmp_stop_bit:X?}"
        )
    }
}

impl PartialEq for Rdh2 {
    fn eq(&self, other: &Self) -> bool {
        self.trigger_type == other.trigger_type
            && self.pages_counter == other.pages_counter
            && self.stop_bit == other.stop_bit
            && self.reserved0 == other.reserved0
    }
}

impl Display for Rdh2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_trigger_type = self.trigger_type;
        let tmp_pages_counter = self.pages_counter;
        let trigger_type_as_hex = format!("{tmp_trigger_type:#x}");
        write!(
            f,
            "{:<10}{:<9}{:<5}",
            trigger_type_as_hex, tmp_pages_counter, self.stop_bit
        )
    }
}

#[repr(packed)]
pub struct Rdh3 {
    pub detector_field: u32, // 23:4 is reserved
    pub par_bit: u16,
    pub reserved0: u16,
}
impl RdhSubWord for Rdh3 {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Rdh3, std::io::Error> {
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

        Ok(Rdh3 {
            detector_field: LittleEndian::read_u32(&load_bytes!(4)),
            par_bit: LittleEndian::read_u16(&load_bytes!(2)),
            reserved0: LittleEndian::read_u16(&load_bytes!(2)),
        })
    }
}
impl PartialEq for Rdh3 {
    fn eq(&self, other: &Self) -> bool {
        self.detector_field == other.detector_field
            && self.par_bit == other.par_bit
            && self.reserved0 == other.reserved0
    }
}
impl Debug for Rdh3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // To align the output, when printing a packed struct, temporary variables are needed
        let tmp_df = self.detector_field;
        let tmp_par = self.par_bit;
        let tmp_res = self.reserved0;
        write!(
            f,
            "Rdh3: detector_field: {tmp_df:x?}, par_bit: {tmp_par:x?}, reserved0: {tmp_res:x?}"
        )
    }
}

impl Display for Rdh3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // To align the output, when printing a packed struct, temporary variables are needed
        let tmp_df = self.detector_field;
        let tmp_par = self.par_bit;
        let tmp_res = self.reserved0;
        write!(
            f,
            "Rdh3: detector_field: {tmp_df:x?}, par_bit: {tmp_par:x?}, reserved0: {tmp_res:x?}"
        )
    }
}
