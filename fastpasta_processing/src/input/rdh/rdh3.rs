//! Struct definition of the `RDH` subword `RDH3`
use super::RdhSubword;
use byteorder::{ByteOrder, LittleEndian};
use std::fmt::{self, Debug, Display};

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
impl RdhSubword for Rdh3 {
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
    use pretty_assertions::assert_eq;

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
