//! Struct definition of the `RDH` subword `RDH0`
use super::RdhSubword;
use byteorder::{ByteOrder, LittleEndian};
use std::fmt::{self, Debug, Display};

/// Represents the composite `FEE ID` fields. Using a newtype because the sub-fields are packed in 16 bits, and extracting the values requires some work.
#[repr(packed)]
#[derive(PartialEq, Default, Clone, Copy)]
pub struct FeeId(pub u16); // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number
                           // Example: L4_12 -> Layer 4 stave 12 = 0b0100_00XX_0000_1100

impl Debug for FeeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tmp_val = self.0;
        write!(f, "{tmp_val}")
    }
}

/// Represents the RDH0 subword of the RDH.
///
/// The RDH0 is 64 bit long.
#[repr(packed)]
#[derive(PartialEq, Debug, Clone, Copy)]
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

    /// Creates a new [RDH0](Rdh0). Subword of the [RDH](super::RdhCru).
    pub const fn new(
        header_id: u8,
        header_size: u8,
        fee_id: FeeId,
        priority_bit: u8,
        system_id: u8,
        reserved0: u16,
    ) -> Self {
        Self {
            header_id,
            header_size,
            fee_id,
            priority_bit,
            system_id,
            reserved0,
        }
    }

    /// Gets the `FEE ID`
    #[inline]
    pub fn fee_id(&self) -> u16 {
        self.fee_id.0
    }
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

impl RdhSubword for Rdh0 {
    #[inline]
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_rdh0_partial_eq() {
        let rdh0 = Rdh0 {
            header_id: 6,
            header_size: 40,
            fee_id: FeeId(0),
            priority_bit: 0,
            system_id: 32,
            reserved0: 0,
        };
        let rdh0_2 = Rdh0 {
            header_id: 6,
            header_size: 40,
            fee_id: FeeId(0),
            priority_bit: 0,
            system_id: 32,
            reserved0: 0,
        };
        assert_eq!(rdh0, rdh0_2);
    }
}
