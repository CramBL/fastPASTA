use super::RdhSubword;
use byteorder::{ByteOrder, LittleEndian};
use std::fmt::{self, Debug, Display};

#[repr(packed)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub(crate) struct FeeId(pub(crate) u16); // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number
                                         // Example: L4_12 -> Layer 4 stave 12 = 0b0100_00XX_0000_1100

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

impl RdhSubword for Rdh0 {
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
