//! Contains struct definitions of the RDH subwords: [RDH0][Rdh0], [RDH1][Rdh1], [RDH2][Rdh2], [RDH3][Rdh3].
// ITS data format: https://gitlab.cern.ch/alice-its-wp10-firmware/RU_mainFPGA/-/wikis/ITS%20Data%20Format#Introduction
use super::lib::RdhSubWord;
use byteorder::{ByteOrder, LittleEndian};
use std::fmt::{self, Debug, Display};

// Newtype pattern used to enforce type safety on fields that are not byte-aligned
#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(packed)]
pub(crate) struct CruidDw(pub(crate) u16); // 12 bit cru_id, 4 bit dw
#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(packed)]
pub(crate) struct BcReserved(pub(crate) u32); // 12 bit bc, 20 bit reserved
#[repr(packed)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub(crate) struct DataformatReserved(pub(crate) u64); // 8 bit data_format, 56 bit reserved0
#[repr(packed)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub(crate) struct FeeId(pub(crate) u16); // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number
                                         // Exaxmple: L4_12 -> Layer 4 stave 12 = 0b0100_00XX_0000_1100

/// Represents the RDH0 subword of the RDH.
///
/// The RDH0 is 64 bit long.
#[repr(packed)]
#[derive(PartialEq, Debug)]
pub struct Rdh0 {
    /// RDH header ID
    pub header_id: u8,
    /// RDH header size
    pub header_size: u8,
    /// RDH FEE ID
    pub(crate) fee_id: FeeId, // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number
    /// RDH priority bit
    pub priority_bit: u8,
    /// RDH system ID
    pub system_id: u8,
    /// RDH reserved0 bits
    pub reserved0: u16,
}

impl Rdh0 {
    /// The side of a [Rdh0] word
    pub const HEADER_SIZE: u8 = 0x40;
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
    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(Rdh0 {
            header_id: buf[0],
            header_size: buf[1],
            fee_id: FeeId(LittleEndian::read_u16(&buf[2..=3])),
            priority_bit: buf[4],
            system_id: buf[5],
            reserved0: LittleEndian::read_u16(&buf[6..=7]),
        })
    }
}

/// Represents the RDH1 subword of the RDH.
///
/// The RDH1 is 64 bit long.
#[repr(packed)]
#[derive(PartialEq, Default, Debug)]
pub struct Rdh1 {
    /// RDH bunch counter 12 bit + reserved 20 bit
    pub(crate) bc_reserved0: BcReserved,
    /// RDH orbit number 32 bits
    pub orbit: u32,
}

impl Rdh1 {
    // only meant for unit tests
    pub(crate) const fn test_new(bc: u16, orbit: u32, reserved0: u32) -> Self {
        Rdh1 {
            bc_reserved0: BcReserved((bc as u32) | (reserved0 << 12)),
            orbit,
        }
    }
    /// Returns the bunch counter.
    pub fn bc(&self) -> u16 {
        (self.bc_reserved0.0 & 0x0FFF) as u16
    }
    /// Returns the reserved bits.
    pub fn reserved0(&self) -> u32 {
        self.bc_reserved0.0 >> 12
    }
}

impl RdhSubWord for Rdh1 {
    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(Rdh1 {
            bc_reserved0: BcReserved(LittleEndian::read_u32(&buf[0..=3])),
            orbit: LittleEndian::read_u32(&buf[4..=7]),
        })
    }
}

impl Display for Rdh1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_orbit = self.orbit;
        let orbit_as_hex = format!("{tmp_orbit:#x}");
        write!(f, "{:<5}{:<12}", self.bc(), orbit_as_hex)
    }
}

/// Represents the RDH2 subword of the RDH.
#[repr(packed)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rdh2 {
    /// RDH trigger type 32 bit.
    pub trigger_type: u32,
    /// RDH pages counter 16 bit.
    pub pages_counter: u16,
    /// RDH stop bit 8 bit.
    pub stop_bit: u8,
    /// RDH reserved 8 bit.
    pub reserved0: u8,
}

impl Rdh2 {
    /// Checks if the 4th bit of the trigger type is set, which indicates that the trigger type is PhT.
    #[inline]
    pub fn is_pht_trigger(&self) -> bool {
        self.trigger_type >> 4 & 0x1 == 1
    }
}

impl RdhSubWord for Rdh2 {
    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(Rdh2 {
            trigger_type: LittleEndian::read_u32(&buf[0..=3]),
            pages_counter: LittleEndian::read_u16(&buf[4..=5]),
            stop_bit: buf[6],
            reserved0: buf[7],
        })
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

/// Represents the RDH3 subword of the RDH.
#[repr(packed)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rdh3 {
    /// RDH detector field 32 bit, but 23:4 are reserved bits.
    pub detector_field: u32,
    /// RDH parity bit 16 bit.
    pub par_bit: u16,
    /// RDH reserved 16 bit.
    pub reserved0: u16,
}
impl RdhSubWord for Rdh3 {
    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(Rdh3 {
            detector_field: LittleEndian::read_u32(&buf[0..=3]),
            par_bit: LittleEndian::read_u16(&buf[4..=5]),
            reserved0: LittleEndian::read_u16(&buf[6..=7]),
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::words;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_rdh0_partial_eq() {
        let rdh0 = Rdh0 {
            header_id: 6,
            header_size: 40,
            fee_id: words::rdh::FeeId(0),
            priority_bit: 0,
            system_id: 32,
            reserved0: 0,
        };
        let rdh0_2 = Rdh0 {
            header_id: 6,
            header_size: 40,
            fee_id: words::rdh::FeeId(0),
            priority_bit: 0,
            system_id: 32,
            reserved0: 0,
        };
        assert_eq!(rdh0, rdh0_2);
    }

    #[test]
    fn test_rdh1_partial_eq() {
        let rdh1 = Rdh1 {
            bc_reserved0: words::rdh::BcReserved(0),
            orbit: 200,
        };
        let rdh1_2 = Rdh1 {
            bc_reserved0: words::rdh::BcReserved(0),
            orbit: 200,
        };
        assert_eq!(rdh1, rdh1_2);
    }

    #[test]
    fn test_rdh2_partial_eq() {
        let rdh2 = Rdh2 {
            trigger_type: 0x00000000,
            pages_counter: 0x0000,
            stop_bit: 0x00,
            reserved0: 0x00,
        };
        let rdh2_2 = rdh2;

        assert_eq!(rdh2, rdh2_2);
    }

    #[test]
    fn test_rdh3_partial_eq() {
        let rdh3 = Rdh3 {
            detector_field: 0x00000000,
            par_bit: 0x0000,
            reserved0: 0x0000,
        };
        println!("{:?}", rdh3);
        let rdh3_2 = rdh3;

        assert_eq!(rdh3, rdh3_2);
    }
}
