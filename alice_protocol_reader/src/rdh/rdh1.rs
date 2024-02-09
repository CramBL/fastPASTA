//! Struct definition of the `RDH` subword `RDH1`
use super::RdhSubword;
use byteorder::{ByteOrder, LittleEndian};
use owo_colors::OwoColorize;
use std::fmt::{self, Debug};

/// Represents the `BC` and `reserved` fields. Using a newtype because the fields are packed in 32 bits, and extracting the values requires some work.
#[repr(packed)]
#[derive(PartialEq, Clone, Copy, Default)]
pub struct BcReserved(pub u32); // 12 bit bc, 20 bit reserved

impl Debug for BcReserved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tmp_val = self.0;
        write!(f, "{tmp_val}")
    }
}

/// Represents the RDH1 subword of the RDH.
///
/// The RDH1 is 64 bit long.
#[repr(packed)]
#[derive(PartialEq, Default, Debug, Clone, Copy)]
pub struct Rdh1 {
    /// RDH bunch counter 12 bit + reserved 20 bit
    pub(crate) bc_reserved0: BcReserved,
    /// RDH orbit number 32 bits
    pub orbit: u32,
}

impl Rdh1 {
    /// Returns the bunch counter.
    #[inline]
    pub fn bc(&self) -> u16 {
        (self.bc_reserved0.0 & 0x0FFF) as u16
    }
    /// Returns the reserved bits.
    #[inline]
    pub fn reserved0(&self) -> u32 {
        self.bc_reserved0.0 >> 12
    }

    /// Valid generic values of a [Rdh1] that can be initialized at constant time
    #[inline]
    pub const fn const_default() -> Self {
        Self {
            bc_reserved0: BcReserved(0),
            orbit: 0,
        }
    }

    /// Make a [Rdh1]
    #[inline]
    pub const fn new(bc_reserved0: BcReserved, orbit: u32) -> Self {
        Self {
            bc_reserved0,
            orbit,
        }
    }
}

impl RdhSubword for Rdh1 {
    #[inline]
    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(Rdh1 {
            bc_reserved0: BcReserved(LittleEndian::read_u32(&buf[0..=3])),
            orbit: LittleEndian::read_u32(&buf[4..=7]),
        })
    }

    fn to_styled_row_view(&self) -> String {
        let tmp_orbit = self.orbit;
        format!(
            "{:<5}{:<12}",
            self.bc().white().bg_rgb::<0, 0, 99>(),
            format!("{:#x}", tmp_orbit).white().bg_rgb::<0, 99, 0>()
        )
    }
}

impl fmt::Display for Rdh1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_orbit = self.orbit;
        let orbit_as_hex = format!("{tmp_orbit:#x}");
        write!(f, "{:<5}{:<12}", self.bc(), orbit_as_hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_rdh1_partial_eq() {
        let rdh1 = Rdh1 {
            bc_reserved0: BcReserved(0),
            orbit: 200,
        };
        let rdh1_2 = Rdh1 {
            bc_reserved0: BcReserved(0),
            orbit: 200,
        };
        assert_eq!(rdh1, rdh1_2);
    }
}
