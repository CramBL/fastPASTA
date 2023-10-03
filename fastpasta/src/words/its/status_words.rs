//! Definitions for status words: [IHW][Ihw], [TDH][Tdh], [TDT][Tdt], [DDW0][Ddw0] & [CDW][Cdw].

use alice_protocol_reader::prelude::macros::load_bytes;
use byteorder::{ByteOrder, LittleEndian};
use std::fmt::{Debug, Display};

impl alice_protocol_reader::prelude::ByteSlice for Ihw {}
impl alice_protocol_reader::prelude::ByteSlice for Tdh {}
impl alice_protocol_reader::prelude::ByteSlice for Cdw {}
impl alice_protocol_reader::prelude::ByteSlice for Tdt {}
impl alice_protocol_reader::prelude::ByteSlice for Ddw0 {}

pub mod util {
    //! Functions for generating human readable text from information extracted from status words.
    //!
    //! These functions takes raw byte slices for performance reasons.
    //! It is crucial that the word is known before being passed to any function here.

    /// Takes a full TDH slice and returns a string description of the trigger field
    pub fn tdh_trigger_as_string(tdh_slice: &[u8]) -> String {
        if tdh_soc_trigger(tdh_slice) {
            String::from("SOC     ")
        } else if tdh_internal_trigger(tdh_slice) {
            String::from("Internal")
        } else if tdh_physics_trigger(tdh_slice) {
            String::from("PhT     ")
        } else {
            String::from("Other   ")
        }
    }

    /// Takes a full TDH slice and returns a string description of the continuation field
    pub fn tdh_continuation_as_string(tdh_slice: &[u8]) -> String {
        debug_assert!(tdh_slice.len() == 10);
        if tdh_continuation(tdh_slice) {
            String::from("Cont.")
        } else {
            String::from("     ")
        }
    }

    /// Takes a DDW0 or TDT slice and returns a 7 char long string description of the most severe lane status among all lanes
    ///
    /// # Examples
    ///
    /// ```
    /// # use fastpasta::words::its::status_words::util::ddw0_tdt_lane_status_as_string;
    /// /// Example of a DDW0 with a all lanes in a OK state
    /// let ddw0_slice = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE4];
    /// assert_eq!(ddw0_tdt_lane_status_as_string(&ddw0_slice), "-      ");
    /// ```
    /// ```
    /// # use fastpasta::words::its::status_words::util::ddw0_tdt_lane_status_as_string;
    /// /// Example of a DDW0 with lane 0 in a Warning state
    /// let ddw0_slice = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE4];
    /// assert_eq!(ddw0_tdt_lane_status_as_string(&ddw0_slice), "Warning");
    /// ```
    /// ```
    /// # use fastpasta::words::its::status_words::util::ddw0_tdt_lane_status_as_string;
    /// /// Example of a TDT with lane 0 in Error state and lane 1 in Fatal state
    /// let tdt_slice = [0b0000_1110, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0];
    /// assert_eq!(ddw0_tdt_lane_status_as_string(&tdt_slice), "Fatal  ");
    /// ```
    pub fn ddw0_tdt_lane_status_as_string(ddw0_tdt_slice: &[u8]) -> String {
        if ddw0_tdt_lane_status_any_fatal(ddw0_tdt_slice) {
            String::from("Fatal  ")
        } else if ddw0_tdt_lane_status_any_error(ddw0_tdt_slice) {
            String::from("Error  ")
        } else if ddw0_tdt_lane_status_any_warning(ddw0_tdt_slice) {
            String::from("Warning")
        } else {
            String::from("-      ")
        }
    }

    /// Takes a full TDH slice and returns a string description of whether the no_data field is 1 or 0
    pub fn tdh_no_data_as_string(tdh_slice: &[u8]) -> String {
        if tdh_no_data(tdh_slice) {
            String::from("No data")
        } else {
            String::from("Data!  ")
        }
    }

    /// Takes a full TDH slice and returns a string description of the trigger orbit and BC in the Orbit_BC format
    pub fn tdh_trigger_orbit_bc_as_string(tdh_slice: &[u8]) -> String {
        // Get the BC from 27:16
        let bc = u16::from_le_bytes([tdh_slice[2], tdh_slice[3]]) & 0x0FFF;
        // Get the Orbit from 63:32
        let orbit = u32::from_le_bytes([tdh_slice[4], tdh_slice[5], tdh_slice[6], tdh_slice[7]]);
        format!("{orbit}_{bc:>4}")
    }

    /// Takes a full TDH slice and returns if the no_data field is set
    pub fn tdh_no_data(tdh_slice: &[u8]) -> bool {
        debug_assert!(tdh_slice.len() == 10);
        tdh_slice[1] & 0b10_0000 != 0
    }
    /// Takes a full TDH slice and returns if continuation bit is set
    pub fn tdh_continuation(tdh_slice: &[u8]) -> bool {
        debug_assert!(tdh_slice.len() == 10);
        tdh_slice[1] & 0b100_0000 != 0
    }

    /// Takes a full TDH slice and returns if the SOC trigger bit [9] is set
    fn tdh_soc_trigger(tdh_slice: &[u8]) -> bool {
        debug_assert!(tdh_slice.len() == 10);
        const SOC_BIT_MASK: u8 = 0b10;
        tdh_slice[1] & SOC_BIT_MASK != 0
    }
    /// Takes a full TDH slice and returns if the internal trigger bit [12] is set
    fn tdh_internal_trigger(tdh_slice: &[u8]) -> bool {
        debug_assert!(tdh_slice.len() == 10);
        tdh_slice[1] & 0b1_0000 != 0
    }
    /// Takes a full TDH slice and returns if the physics trigger bit [4] is set
    fn tdh_physics_trigger(tdh_slice: &[u8]) -> bool {
        debug_assert!(tdh_slice.len() == 10);
        tdh_slice[0] & 0b1_0000 != 0
    }

    /// Takes a full TDT slice and returns if packet_done bit is set
    pub fn tdt_packet_done(tdt_slice: &[u8]) -> bool {
        debug_assert!(tdt_slice.len() == 10);
        tdt_slice[8] & 0b1 != 0
    }

    /// Takes a full TDT slice and returns a string description of whether the packet_done bit is set
    pub fn tdt_packet_done_as_string(tdt_slice: &[u8]) -> String {
        debug_assert!(tdt_slice.len() == 10);
        if tdt_packet_done(tdt_slice) {
            String::from("Complete")
        } else {
            String::from("Split   ")
        }
    }

    /// Takes a DDW0 slice and returns true if any lanes status is not OK
    #[allow(dead_code)]
    fn ddw0_lane_status_not_ok(ddw0_slice: &[u8]) -> bool {
        debug_assert!(ddw0_slice.len() == 10);
        let first_7_bytes = &ddw0_slice[..7];
        first_7_bytes.iter().any(|byte| *byte != 0)
    }

    /// Takes a DDW0 slice and returns true if any lanes status is warning
    fn ddw0_tdt_lane_status_any_warning(ddw0_slice: &[u8]) -> bool {
        debug_assert!(ddw0_slice.len() == 10);
        const LANE_WARNING_MASK: u8 = 0b0101_0101;
        let first_7_bytes = &ddw0_slice[..7];
        first_7_bytes
            .iter()
            .any(|byte| *byte & LANE_WARNING_MASK != 0)
    }

    /// Takes a DDW0 slice and returns true if any lanes status is error
    fn ddw0_tdt_lane_status_any_error(ddw0_slice: &[u8]) -> bool {
        debug_assert!(ddw0_slice.len() == 10);
        const LANE_ERROR_MASK: u8 = 0b1010_1010;
        let first_7_bytes = &ddw0_slice[..7];
        first_7_bytes
            .iter()
            .any(|byte| *byte & LANE_ERROR_MASK != 0)
    }

    /// Takes a DDW0 slice and returns true if any lanes status is fatal
    fn ddw0_tdt_lane_status_any_fatal(ddw0_slice: &[u8]) -> bool {
        debug_assert!(ddw0_slice.len() == 10);
        const LANE_FATAL_MASK0: u8 = 0b0000_0011;
        const LANE_FATAL_MASK1: u8 = 0b0000_1100;
        const LANE_FATAL_MASK2: u8 = 0b0011_0000;
        const LANE_FATAL_MASK3: u8 = 0b1100_0000;
        let first_7_bytes = &ddw0_slice[..7];
        first_7_bytes.iter().any(|byte| {
            *byte & LANE_FATAL_MASK0 == LANE_FATAL_MASK0
                || *byte & LANE_FATAL_MASK1 == LANE_FATAL_MASK1
                || *byte & LANE_FATAL_MASK2 == LANE_FATAL_MASK2
                || *byte & LANE_FATAL_MASK3 == LANE_FATAL_MASK3
        })
    }

    /// Takes a DDW0 slice and returns if the lane_starts_violation bit [67] is set
    #[allow(dead_code)]
    fn ddw0_lane_starts_violation(ddw0_slice: &[u8]) -> bool {
        debug_assert!(ddw0_slice.len() == 10);
        ddw0_slice[8] & 0b1000 != 0
    }

    /// Takes a DDW0 slice and returns if the transmission timeout bit [65] is set
    #[allow(dead_code)]
    fn ddw0_transmission_timeout(ddw0_slice: &[u8]) -> bool {
        debug_assert!(ddw0_slice.len() == 10);
        ddw0_slice[8] & 0b10 != 0
    }
}

/// Trait to implement for all status words
pub trait StatusWord:
    std::fmt::Debug + PartialEq + Sized + alice_protocol_reader::prelude::ByteSlice + Display
{
    /// Returns the id of the status word
    fn id(&self) -> u8;
    /// Deserializes the status word from a reader and a byte slice
    #[inline]
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let buf = load_bytes!(10, reader);
        Self::from_buf(&buf)
    }
    /// Deserializes the GBT word from a byte slice
    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error>;
    /// Sanity check that returns true if all reserved bits are 0
    fn is_reserved_0(&self) -> bool;
}

/// Helper to display all the status words in a similar way, without dynamic dispatch
#[inline]
fn display_byte_slice<T: StatusWord>(
    status_word: &T,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let slice = status_word.to_byte_slice();
    write!(
        f,
        "{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
        slice[0],
        slice[1],
        slice[2],
        slice[3],
        slice[4],
        slice[5],
        slice[6],
        slice[7],
        slice[8],
        slice[9],
    )
}

/// Checks if the corrosponding lane bit is set in the IHW active lanes field
pub fn is_lane_active(lane: u8, active_lanes: u32) -> bool {
    //log::debug!("Lane: {lane}, Active lanes: {active_lanes:#X}");
    let lane = lane as u32;
    let mask = 1 << lane;
    active_lanes & mask != 0
}
/// Struct to represent the IHW status word
#[repr(packed)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ihw {
    // Total of 80 bits
    // ID: 0xE0
    active_lanes: u32, // 27:0
    reserved: u32,     // 71:28
    id: u16,           // 79:72
}

impl Ihw {
    /// Returns the integer value of the reserved bits
    pub fn reserved(&self) -> u64 {
        let four_lsb: u8 = ((self.active_lanes >> 28) & 0xF) as u8;
        let eight_msb = self.id & 0xFF;
        (eight_msb as u64) << 36 | (self.reserved as u64) << 4 | (four_lsb as u64)
    }
    /// Returns the integer value of the active lanes field
    pub fn active_lanes(&self) -> u32 {
        self.active_lanes & 0xFFFFFFF
    }
}

impl Display for Ihw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_byte_slice(self, f)
    }
}

impl StatusWord for Ihw {
    fn id(&self) -> u8 {
        (self.id >> 8) as u8
    }

    fn is_reserved_0(&self) -> bool {
        self.reserved() == 0
    }

    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(Ihw {
            active_lanes: LittleEndian::read_u32(&buf[0..=3]),
            reserved: LittleEndian::read_u32(&buf[4..=7]),
            id: LittleEndian::read_u16(&buf[8..=9]),
        })
    }
}

/// Struct to represent the TDH status word
#[repr(packed)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Tdh {
    // 11:0 trigger_type
    // 12: internal_trigger, 13: no_data, 14: continuation, 15: reserved
    trigger_type_internal_trigger_no_data_continuation_reserved2: u16,
    trigger_bc_reserved1: u16,     // 27:16 trigger_bc, 31:28 reserved,
    pub(crate) trigger_orbit: u32, // 63:32
    // ID 0xe8
    reserved0_id: u16, // 71:64 reserved, 79:72 id
}
impl Tdh {
    /// Maximum value of the trigger_bc field
    pub const MAX_BC: u16 = 3563;
    /// Returns the integer value of the reserved0 field
    pub fn reserved0(&self) -> u16 {
        self.reserved0_id & 0xFF
    }

    /// Returns the integer value of the reserved1 field
    pub fn reserved1(&self) -> u16 {
        self.trigger_bc_reserved1 & 0xF000 // doesn't need shift as it should just be checked if equal to 0
    }

    /// Returns the integer value of the trigger_bc field
    pub fn trigger_bc(&self) -> u16 {
        self.trigger_bc_reserved1 & 0x0FFF
    }

    /// Returns the integer value of the reserved2 field
    pub fn reserved2(&self) -> u16 {
        // 15th bit is reserved
        self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b1000_0000_0000_0000
    }

    /// Returns the integer value of the continuation field
    pub fn continuation(&self) -> u16 {
        // 14th bit is continuation
        (self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b100_0000_0000_0000)
            >> 14
    }

    /// Returns the integer value of the no_data field
    pub fn no_data(&self) -> u16 {
        // 13th bit is no_data
        (self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b10_0000_0000_0000)
            >> 13
    }

    /// Returns the integer value of the internal_trigger field
    pub fn internal_trigger(&self) -> u16 {
        // 12th bit is internal_trigger
        (self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b1_0000_0000_0000)
            >> 12
    }

    /// Returns the integer value of the trigger_type field
    ///
    /// Beware! Only 12 LSB are valid!
    pub fn trigger_type(&self) -> u16 {
        // 11:0 is trigger_type
        self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b1111_1111_1111
    }
}

impl Display for Tdh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_byte_slice(self, f)
    }
}

impl StatusWord for Tdh {
    fn id(&self) -> u8 {
        (self.reserved0_id >> 8) as u8
    }

    fn is_reserved_0(&self) -> bool {
        self.reserved0() == 0 && self.reserved1() == 0 && self.reserved2() == 0
    }

    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(Tdh {
            trigger_type_internal_trigger_no_data_continuation_reserved2: LittleEndian::read_u16(
                &buf[0..=1],
            ),
            trigger_bc_reserved1: LittleEndian::read_u16(&buf[2..=3]),
            trigger_orbit: LittleEndian::read_u32(&buf[4..=7]),
            reserved0_id: LittleEndian::read_u16(&buf[8..=9]),
        })
    }
}

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

impl Display for Ddw0 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(Self {
            res3_lane_status: LittleEndian::read_u64(&buf[0..=7]),
            index: buf[8],
            id: buf[9],
        })
    }
}

/// Struct representing the CDW.
#[repr(packed)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Cdw {
    calibration_word_index_lsb_calibration_user_fields: u64, // 63:48 calibration_word_index_LSB 47:0 calibration_user_fields
    calibration_word_index_msb: u8,                          // 71:64 calibration_word_index_MSB
    // ID: 0xF8
    id: u8,
}

impl Cdw {
    /// Returns the integer value of the calibration_word_index field.
    pub fn calibration_word_index(&self) -> u32 {
        ((self.calibration_word_index_msb as u32) << 16)
            | ((self.calibration_word_index_lsb_calibration_user_fields >> 48) as u32)
    }
    /// Returns the integer value of the calibration_user_fields field.
    pub fn calibration_user_fields(&self) -> u64 {
        self.calibration_word_index_lsb_calibration_user_fields & 0xffff_ffff_ffff
    }
}

impl Display for Cdw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_byte_slice(self, f)
    }
}

impl StatusWord for Cdw {
    fn id(&self) -> u8 {
        self.id
    }

    fn is_reserved_0(&self) -> bool {
        true // No reserved bits
    }

    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(Self {
            calibration_word_index_lsb_calibration_user_fields: LittleEndian::read_u64(&buf[0..=7]),
            calibration_word_index_msb: buf[8],
            id: buf[9],
        })
    }
}

#[cfg(test)]
mod tests {
    use alice_protocol_reader::prelude::ByteSlice;

    use super::{util::*, *};
    use pretty_assertions::assert_eq;

    #[test]
    fn ihw_read_write() {
        const VALID_ID: u8 = 0xE0;
        const ACTIVE_LANES_14_ACTIVE: u32 = 0x3F_FF;
        let raw_data_ihw = [0xFF, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0];
        let ihw = Ihw::load(&mut raw_data_ihw.as_slice()).unwrap();
        assert_eq!(ihw.id(), VALID_ID);
        assert!(ihw.is_reserved_0());
        assert_eq!(ihw.active_lanes(), ACTIVE_LANES_14_ACTIVE);
        println!("{ihw:#?}");
        let loaded_ihw = Ihw::load(&mut ihw.to_byte_slice()).unwrap();
        println!("{loaded_ihw:?}");
        assert_eq!(ihw, loaded_ihw);
    }

    #[test]
    fn tdh_read_write() {
        const VALID_ID: u8 = 0xE8;
        let raw_data_tdh = [0x03, 0x1A, 0x00, 0x00, 0x75, 0xD5, 0x7D, 0x0B, 0x00, 0xE8];
        const TRIGGER_TYPE: u16 = 0xA03;
        const INTERNAL_TRIGGER: u16 = 1; // 0x1
        const NO_DATA: u16 = 0; // 0x0
        const CONTINUATION: u16 = 0; // 0x0
        const TRIGGER_BC: u16 = 0;
        const TRIGGER_ORBIT: u32 = 0x0B7DD575;
        let tdh = Tdh::load(&mut raw_data_tdh.as_slice()).unwrap();
        println!("{tdh}");
        assert_eq!(tdh.id(), VALID_ID);
        assert!(tdh.is_reserved_0());
        assert_eq!(tdh.trigger_type(), TRIGGER_TYPE);
        assert_eq!(tdh.internal_trigger(), INTERNAL_TRIGGER);
        assert_eq!(tdh.no_data(), NO_DATA);
        assert_eq!(tdh.continuation(), CONTINUATION);
        assert_eq!(tdh.trigger_bc(), TRIGGER_BC);
        let trigger_orbit = tdh.trigger_orbit;
        assert_eq!(trigger_orbit, TRIGGER_ORBIT);
        let loaded_tdh = Tdh::load(&mut tdh.to_byte_slice()).unwrap();
        assert_eq!(tdh, loaded_tdh);
    }

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

    #[test]
    fn cdw_read_write() {
        const VALID_ID: u8 = 0xF8;
        let raw_data_cdw = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0xF8];
        assert!(raw_data_cdw[9] == VALID_ID);
        let cdw = Cdw::load(&mut raw_data_cdw.as_slice()).unwrap();
        println!("{cdw:#?}");
        assert_eq!(cdw.id(), VALID_ID);
        assert!(cdw.is_reserved_0());
        assert_eq!(cdw.calibration_user_fields(), 0x050403020100);
        assert_eq!(cdw.calibration_word_index(), 0x080706);
        let loaded_cdw = Cdw::load(&mut cdw.to_byte_slice()).unwrap();
        assert_eq!(cdw, loaded_cdw);
    }

    #[test]
    fn test_tdh_trigger_as_string() {
        let tdh_slice = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE8];

        let trig_as_string = tdh_trigger_as_string(&tdh_slice);

        assert_eq!(trig_as_string, "Other   ");
    }

    #[test]
    fn test_tdh_continuation_as_string() {
        // Only continuation flag set other than ID
        let tdh_slice = [
            0x00,
            0b0100_0000, // Continuation flag set
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0xE8,
        ];

        let cont_as_string = tdh_continuation_as_string(&tdh_slice);

        assert_eq!(cont_as_string, "Cont.");
    }
}
