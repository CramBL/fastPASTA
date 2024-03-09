//! Struct definition of the `RDH` subword `RDH3`
use super::RdhSubword;
use byteorder::{ByteOrder, LittleEndian};
use std::fmt::{self, Debug, Display};

/// Represents the RDH3 subword of the [RDH](super::RdhCru).
#[repr(packed)]
#[derive(Clone, PartialEq, Debug, Copy)]
pub struct Rdh3 {
    /// RDH detector field 32 bit, but as of v1.21.0 23:12 are reserved bits.
    pub detector_field: u32,
    /// RDH parity bit 16 bit.
    pub par_bit: u16,
    /// RDH reserved 16 bit.
    pub reserved0: u16,
}

impl Rdh3 {
    /// Creates a new [RDH3](Rdh3). Subword of the [RDH](super::RdhCru).
    pub const fn new(detector_field: u32, par_bit: u16, reserved0: u16) -> Self {
        Self {
            detector_field,
            par_bit,
            reserved0,
        }
    }
}

impl RdhSubword for Rdh3 {
    #[inline]
    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(Rdh3 {
            detector_field: LittleEndian::read_u32(&buf[0..=3]),
            par_bit: LittleEndian::read_u16(&buf[4..=5]),
            reserved0: LittleEndian::read_u16(&buf[6..=7]),
        })
    }

    fn to_styled_row_view(&self) -> String {
        let tmp_df = self.detector_field;
        let tmp_par = self.par_bit;
        let tmp_res = self.reserved0;
        format!("{:<10}{:<9}{:<5}", format!("{tmp_df:#x}"), tmp_par, tmp_res)
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

pub mod det_field_util {
    //! Utility for making sense of the detector field
    //!
    //! [ITS Data Format](https://gitlab.cern.ch/alice-its-wp10-firmware/RU_mainFPGA/-/wikis/ITS-Data-Format#RDHDetectorField)
    #![allow(dead_code)]

    // ([31:27] Expert reserved] - Non-zero indicates error)

    /// `[26]`: Clock event
    pub fn clock_event(det_field: u32) -> bool {
        det_field & 0b100_0000_0000_0000_0000_0000_0000 != 0
    }

    /// `[25]`: Timebase event
    pub fn timebase_event(det_field: u32) -> bool {
        det_field & 0b10_0000_0000_0000_0000_0000_0000 != 0
    }

    /// `[24]`: Timebase unsynced
    pub fn timebase_unsynced(det_field: u32) -> bool {
        det_field & 0b1_0000_0000_0000_0000_0000_0000 != 0
    }

    // (Reserved 23:12)

    /// `[11:6]`: User configurable reserved
    ///
    /// Can be set to configurable value by register (v1.21.0)
    /// returns an 8-bit value representing the value of the 6-bits that signify the user configurable reserved bits
    pub fn user_configurable_reserved(det_field: u32) -> u8 {
        ((det_field & 0b1111_1100_0000) >> 6) as u8
    }

    /// `[5]`: Stave autorecovery including trigger ramp
    ///
    /// User configurable: to be set manually during the autorecovery including a trigger ramping (v1.21.0)
    pub fn stave_autorecovery_including_trigger_ramp(det_field: u32) -> bool {
        det_field & 0b10_0000 != 0
    }

    /// `[4]`: Trigger ramp
    ///
    /// User configurable: to be set manually during trigger ramping (v1.21.0)
    pub fn trigger_ramp(det_field: u32) -> bool {
        det_field & 0b1_0000 != 0
    }

    /// `[3]`: Lane FATAL
    ///
    /// Set if at least one lane had a FATAL in this HBF, i.e. lane has status as FATAL in DDW0
    pub fn lane_fatal(det_field: u32) -> bool {
        det_field & 0b1000 != 0
    }

    /// `[2]`: Lane ERROR
    ///
    /// Set if at least one lane had an ERROR in this HBF, i.e. lane has status as ERROR in DDW0
    pub fn lane_error(det_field: u32) -> bool {
        det_field & 0b100 != 0
    }

    /// `[1]`: Lane WARNING
    ///
    /// Set if at least one lane had a WARNING in this HBF, i.e. lane has status as WARNING in DDW0
    pub fn lane_warning(det_field: u32) -> bool {
        det_field & 0b10 != 0
    }

    /// `[0]`: Lane missing data
    ///
    /// DDW0 follows (if this is a STOP RDH) with per-lane details of lane status (strip/error/fatal). Expected to be 1 if any of bit `[3:1]` is set.
    pub fn lane_missing_data(det_field: u32) -> bool {
        det_field & 0b1 != 0
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
        let rdh3_2 = rdh3;

        assert_eq!(rdh3, rdh3_2);
    }

    #[test]
    fn test_rdh3_to_string() {
        let rdh3 = Rdh3::from_buf(&[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

        assert_eq!(
            rdh3.to_string(),
            "Rdh3: detector_field: 4030201, par_bit: 605, reserved0: 807"
        );
    }
}
