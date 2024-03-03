//! Contains the struct definition of the DDW0

use super::display_byte_slice;
use crate::util::*;

/// Struct representing the DDW0.
#[repr(packed)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ddw0 {
    // 64:56 reserved0, 55:0 lane_status
    res3_lane_status: u64,
    // 71:68 index, 67: lane_starts_violation, 66: reserved0, 65: transmission_timeout, 64: reserved1
    index: u8,
    // ID: 0xe4
    id: u8, // 79:72
}

impl Ddw0 {
    /// Returns the integer value of the index field.
    pub fn index(&self) -> u8 {
        (self.index & 0xF0) >> 4
    }
    /// Returns true if the lane_starts_violation bit is set.
    pub fn lane_starts_violation(&self) -> bool {
        (self.index & 0b1000) != 0
    }
    /// Returns true if the transmission_timeout bit is set.
    pub fn transmission_timeout(&self) -> bool {
        (self.index & 0b10) != 0
    }
    /// Returns the integer value of the lane_status field.
    pub fn lane_status(&self) -> u64 {
        self.res3_lane_status & 0x00ff_ffff_ffff_ffff
    }
    /// Returns the 2 reserved bits 66 & 64 in position 2 & 0.
    pub fn reserved0_1(&self) -> u8 {
        self.index & 0b0000_0101
    }
    /// Returns the 8 reserved bits 64:56 in position 7:0.
    pub fn reserved2(&self) -> u8 {
        ((self.res3_lane_status & 0xFF00_0000_0000_0000) >> 56) as u8
    }
}

impl fmt::Display for Ddw0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_byte_slice(self, f)
    }
}

impl StatusWord for Ddw0 {
    fn id(&self) -> u8 {
        self.id
    }

    fn is_reserved_0(&self) -> bool {
        (self.index & 0b0000_0101) == 0 && (self.res3_lane_status & 0xFF00_0000_0000_0000) == 0
    }

    fn from_buf(buf: &[u8]) -> Result<Self, io::Error> {
        Ok(Self {
            res3_lane_status: LittleEndian::read_u64(&buf[0..=7]),
            index: buf[8],
            id: buf[9],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn ddw0_read_write() {
        const VALID_ID: u8 = 0xE4;
        let raw_data_ddw0 = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE4];
        assert!(raw_data_ddw0[9] == VALID_ID);
        let ddw0 = Ddw0::load(&mut raw_data_ddw0.as_slice()).unwrap();

        assert_eq!(ddw0.id(), VALID_ID);
        assert!(ddw0.is_reserved_0());
        assert!(!ddw0.transmission_timeout());
        assert!(!ddw0.lane_starts_violation());
        assert_eq!(ddw0.lane_status(), 0);
        let loaded_ddw0 = Ddw0::load(&mut ddw0.to_byte_slice()).unwrap();
        assert_eq!(ddw0, loaded_ddw0);
        println!("{ddw0}");
    }

    #[test]
    fn ddw0_reporting_errors_read_write() {
        const VALID_ID: u8 = 0xE4;
        // Atypical TDT, some lane errors and warnings etc.
        const LANE_0_AND_3_IN_WARNING: u8 = 0b0100_0001;
        const LANE_4_TO_7_IN_FATAL: u8 = 0b1111_1111;
        const LANE_8_TO_11_IN_WARNING: u8 = 0b0101_0101;
        const LANE_12_AND_15_IN_ERROR: u8 = 0b1000_0010;
        const LANE_16_AND_19_IN_OK: u8 = 0b0000_0000;
        const LANE_22_IN_WARNING: u8 = 0b0001_0000;
        const LANE_24_AND_25_IN_ERROR: u8 = 0b0000_1010;
        const RESERVED0: u8 = 0x00;
        const TRANSMISSION_TO_LANE_STARTS_VIOLATION_SET: u8 = 0x0A;

        let raw_data_ddw0 = [
            LANE_0_AND_3_IN_WARNING,
            LANE_4_TO_7_IN_FATAL,
            LANE_8_TO_11_IN_WARNING,
            LANE_12_AND_15_IN_ERROR,
            LANE_16_AND_19_IN_OK,
            LANE_22_IN_WARNING,
            LANE_24_AND_25_IN_ERROR,
            RESERVED0,
            TRANSMISSION_TO_LANE_STARTS_VIOLATION_SET,
            0xE4,
        ];
        assert_eq!(raw_data_ddw0[9], VALID_ID);
        let ddw0 = Ddw0::load(&mut raw_data_ddw0.as_slice()).unwrap();
        println!("{ddw0}");
        assert_eq!(ddw0.id(), VALID_ID);

        assert!(ddw0.index() == 0);
        assert!(ddw0.is_reserved_0());
        assert!(ddw0.transmission_timeout());
        assert!(ddw0.lane_starts_violation());
        let combined_lane_status: u64 = ((LANE_24_AND_25_IN_ERROR as u64) << 48)
            | ((LANE_22_IN_WARNING as u64) << 40)
            | ((LANE_16_AND_19_IN_OK as u64) << 32)
            | ((LANE_12_AND_15_IN_ERROR as u64) << 24)
            | ((LANE_8_TO_11_IN_WARNING as u64) << 16)
            | ((LANE_4_TO_7_IN_FATAL as u64) << 8)
            | (LANE_0_AND_3_IN_WARNING as u64);
        println!("{combined_lane_status:x}");
        assert_eq!(ddw0.lane_status(), combined_lane_status);
        let loaded_ddw0 = Ddw0::load(&mut ddw0.to_byte_slice()).unwrap();
        assert_eq!(ddw0, loaded_ddw0);
    }
}
