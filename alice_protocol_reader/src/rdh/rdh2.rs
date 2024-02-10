//! Struct definition of the `RDH` subword `RDH2`
use crate::prelude::{BLUE, GREEN};

use super::RdhSubword;
use byteorder::{ByteOrder, LittleEndian};
use owo_colors::OwoColorize;
use std::fmt::{self, Debug, Display};

/// Represents the `RDH2` subword of the [RDH](super::RdhCru).
#[repr(packed)]
#[derive(Clone, PartialEq, Debug, Copy)]
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

    /// Creates a new [RDH2](Rdh2). Subword of the [RDH](super::RdhCru).
    pub const fn new(trigger_type: u32, pages_counter: u16, stop_bit: u8, reserved0: u8) -> Self {
        Self {
            trigger_type,
            pages_counter,
            stop_bit,
            reserved0,
        }
    }
}

impl RdhSubword for Rdh2 {
    #[inline]
    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(Rdh2 {
            trigger_type: LittleEndian::read_u32(&buf[0..=3]),
            pages_counter: LittleEndian::read_u16(&buf[4..=5]),
            stop_bit: buf[6],
            reserved0: buf[7],
        })
    }

    fn to_styled_row_view(&self) -> String {
        let tmp_trigger_type = self.trigger_type;
        let tmp_pages_counter = self.pages_counter;
        let trigger_type_as_hex = format!("{tmp_trigger_type:#x}");
        format!(
            "{:<10}{:<9}{}",
            trigger_type_as_hex.white().bg_rgb::<0, GREEN, 0>(),
            tmp_pages_counter.white().bg_rgb::<0, 0, BLUE>(),
            format_args!("{:<5} ", self.stop_bit)
                .white()
                .bg_rgb::<0, GREEN, 0>()
        )
    }
}

impl Display for Rdh2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_trigger_type = self.trigger_type;
        let tmp_pages_counter = self.pages_counter;
        let trigger_type_as_hex = format!("{tmp_trigger_type:#x}");
        write!(
            f,
            "{:<10}{:<9}{:<5} ",
            trigger_type_as_hex, tmp_pages_counter, self.stop_bit
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

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
}
