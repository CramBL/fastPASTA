//! Contains the struct definition of the TDT

use std::fmt::Display;

use byteorder::{ByteOrder, LittleEndian};

use super::{display_byte_slice, StatusWord};

/// Struct representing the TDT
#[repr(packed)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Tdt {
    // 55:0 lane_status
    lane_status_15_0: u32,
    lane_status_23_16: u16,
    lane_status_27_24: u8,
    // 63: timeout_to_start, 62: timeout_start_stop, 61: timeout_in_idle, 60:56 Reserved
    timeout_to_start_timeout_start_stop_timeout_in_idle_res2: u8,

    // 71:68 reserved, 67: lane_starts_violation, 66: reserved, 65: transmission_timeout, 64: packet_done
    res0_lane_starts_violation_res1_transmission_timeout_packet_done: u8,
    // ID 0xf0
    id: u8,
}

impl Tdt {
    /// Returns the integer value of the reserved0 field.
    pub fn reserved0(&self) -> u8 {
        self.res0_lane_starts_violation_res1_transmission_timeout_packet_done >> 4
    }
    /// Returns true if the lane_starts_violation bit is set.
    pub fn lane_starts_violation(&self) -> bool {
        (self.res0_lane_starts_violation_res1_transmission_timeout_packet_done & 0b1000) != 0
    }
    /// Returns the integer value of the reserved1 field.
    pub fn reserved1(&self) -> u8 {
        self.res0_lane_starts_violation_res1_transmission_timeout_packet_done & 0b0100
    }
    /// Returns true if the transmission_timeout bit is set.
    pub fn transmission_timeout(&self) -> bool {
        (self.res0_lane_starts_violation_res1_transmission_timeout_packet_done & 0b0010) != 0
    }
    /// Returns true if the packet_done bit is set.
    pub fn packet_done(&self) -> bool {
        (self.res0_lane_starts_violation_res1_transmission_timeout_packet_done & 0b0001) == 1
    }
    /// Returns true if the timeout_to_start bit is set.
    pub fn timeout_to_start(&self) -> bool {
        (self.timeout_to_start_timeout_start_stop_timeout_in_idle_res2 & 0b1000_0000) != 0
    }
    /// Returns true if the timeout_start_stop bit is set.
    pub fn timeout_start_stop(&self) -> bool {
        (self.timeout_to_start_timeout_start_stop_timeout_in_idle_res2 & 0b0100_0000) != 0
    }
    /// Returns true if the timeout_in_idle bit is set.
    pub fn timeout_in_idle(&self) -> bool {
        (self.timeout_to_start_timeout_start_stop_timeout_in_idle_res2 & 0b0010_0000) != 0
    }
    /// Returns the integer value of the reserved2 field.
    pub fn reserved2(&self) -> u8 {
        self.timeout_to_start_timeout_start_stop_timeout_in_idle_res2 & 0b0001_1111
    }
    /// Returns the integer value of bits \[55:48\] of the lane_status field, corresponding to the status of lanes 27-24.
    pub fn lane_status_27_24(&self) -> u8 {
        self.lane_status_27_24
    }
    /// Returns the integer value of bits \[47:32\] of the lane_status field, corresponding to the status of lanes 23-16.
    pub fn lane_status_23_16(&self) -> u16 {
        self.lane_status_23_16
    }
    /// Returns the integer value of bits \[31:0\] of the lane_status field, corresponding to the status of lanes 15-0.
    pub fn lane_status_15_0(&self) -> u32 {
        self.lane_status_15_0
    }
}

impl Display for Tdt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_byte_slice(self, f)
    }
}
impl StatusWord for Tdt {
    fn id(&self) -> u8 {
        self.id
    }

    fn is_reserved_0(&self) -> bool {
        self.reserved0() == 0 && self.reserved1() == 0 && self.reserved2() == 0
    }

    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(Self {
            lane_status_15_0: LittleEndian::read_u32(&buf[0..=3]),
            lane_status_23_16: LittleEndian::read_u16(&buf[4..=5]),
            lane_status_27_24: buf[6],
            timeout_to_start_timeout_start_stop_timeout_in_idle_res2: buf[7],
            res0_lane_starts_violation_res1_transmission_timeout_packet_done: buf[8],
            id: buf[9],
        })
    }
}

#[cfg(test)]
mod tests {
    use alice_protocol_reader::prelude::ByteSlice;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn tdt_read_write() {
        const VALID_ID: u8 = 0xF0;
        // Boring but very typical TDT, everything is 0 except for packet_done
        let raw_data_tdt = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xF0];
        assert_eq!(raw_data_tdt[9], VALID_ID);
        let tdt = Tdt::load(&mut raw_data_tdt.as_slice()).unwrap();
        println!("{tdt}");
        assert_eq!(tdt.id(), VALID_ID);
        assert!(tdt.is_reserved_0());
        assert!(tdt.packet_done());
        let loaded_tdt = Tdt::load(&mut tdt.to_byte_slice()).unwrap();
        assert_eq!(tdt, loaded_tdt);
    }

    #[test]
    fn tdt_reporting_errors_read_write() {
        const VALID_ID: u8 = 0xF0;
        // Atypical TDT, some lane errors and warnings etc.
        const LANE_0_AND_3_IN_WARNING: u8 = 0b0100_0001;
        const LANE_4_TO_7_IN_FATAL: u8 = 0b1111_1111;
        const LANE_8_TO_11_IN_WARNING: u8 = 0b0101_0101;
        const LANE_12_AND_15_IN_ERROR: u8 = 0b1000_0010;
        const LANE_16_AND_19_IN_OK: u8 = 0b0000_0000;
        const LANE_22_IN_WARNING: u8 = 0b0001_0000;
        const LANE_24_AND_25_IN_ERROR: u8 = 0b0000_1010;
        const TIMEOUT_TO_START_TIMEOUT_START_STOP_TIMEOUT_IN_IDLE_ALL_SET: u8 = 0xE0;
        const LANE_STARTS_VIOLATION_AND_TRANSMISSION_TIMEOUT_SET: u8 = 0x0A;

        let raw_data_tdt = [
            LANE_0_AND_3_IN_WARNING,
            LANE_4_TO_7_IN_FATAL,
            LANE_8_TO_11_IN_WARNING,
            LANE_12_AND_15_IN_ERROR,
            LANE_16_AND_19_IN_OK,
            LANE_22_IN_WARNING,
            LANE_24_AND_25_IN_ERROR,
            TIMEOUT_TO_START_TIMEOUT_START_STOP_TIMEOUT_IN_IDLE_ALL_SET,
            LANE_STARTS_VIOLATION_AND_TRANSMISSION_TIMEOUT_SET,
            0xF0,
        ];
        assert!(raw_data_tdt[9] == VALID_ID);
        let tdt = Tdt::load(&mut raw_data_tdt.as_slice()).unwrap();
        println!("{tdt}");
        assert_eq!(tdt.id(), VALID_ID);
        println!("tdt.is_reserved_0() = {}", tdt.is_reserved_0());
        println!(
            "{:x} {:x} {:x}",
            tdt.reserved0(),
            tdt.reserved1(),
            tdt.reserved2()
        );
        assert!(tdt.is_reserved_0());
        assert!(!tdt.packet_done());
        assert!(tdt.transmission_timeout());
        assert!(tdt.lane_starts_violation());
        assert!(tdt.timeout_to_start());
        assert!(tdt.timeout_start_stop());
        assert!(tdt.timeout_in_idle());
        assert_eq!(tdt.lane_status_27_24(), LANE_24_AND_25_IN_ERROR);
        let combined_lane_status_23_to_16 =
            ((LANE_22_IN_WARNING as u16) << 8) | (LANE_16_AND_19_IN_OK as u16);
        assert_eq!(tdt.lane_status_23_16(), combined_lane_status_23_to_16);
        let combined_lane_status_15_to_0 = ((LANE_12_AND_15_IN_ERROR as u32) << 24)
            | ((LANE_8_TO_11_IN_WARNING as u32) << 16)
            | ((LANE_4_TO_7_IN_FATAL as u32) << 8)
            | (LANE_0_AND_3_IN_WARNING as u32);
        assert_eq!(tdt.lane_status_15_0(), combined_lane_status_15_to_0);

        let loaded_tdt = Tdt::load(&mut tdt.to_byte_slice()).unwrap();
        assert_eq!(tdt, loaded_tdt);
        let tdt_ref = &tdt;
        println!("{tdt_ref}");
    }
}
