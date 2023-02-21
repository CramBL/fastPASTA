use crate::ByteSlice;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::fmt::Debug;

pub trait StatusWord: std::fmt::Debug + PartialEq + Sized + ByteSlice {
    fn id(&self) -> u8;
    fn print(&self);
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn is_reserved_0(&self) -> bool;
}

#[repr(packed)]
pub struct Ihw {
    // Total of 80 bits
    // ID: 0xE0
    active_lanes: u32, // 27:0
    reserved: u32,     // 71:28
    id: u16,           // 79:72
}

impl Ihw {
    pub fn reserved(&self) -> u64 {
        let four_lsb: u8 = ((self.active_lanes >> 28) & 0xF) as u8;
        let eight_msb = self.id & 0xFF;
        (eight_msb as u64) << 36 | (self.reserved as u64) << 4 | (four_lsb as u64)
    }
    pub fn active_lanes(&self) -> u32 {
        self.active_lanes & 0xFFFFFFF
    }
}

impl StatusWord for Ihw {
    fn id(&self) -> u8 {
        (self.id >> 8) as u8
    }
    fn print(&self) {
        println!(
            "IHW: {:x} {:x} {:x}",
            self.id(),
            self.reserved(),
            self.active_lanes()
        );
    }

    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(Ihw {
            active_lanes: reader.read_u32::<LittleEndian>().unwrap(),
            reserved: reader.read_u32::<LittleEndian>().unwrap(),
            id: reader.read_u16::<LittleEndian>().unwrap(),
        })
    }
    fn is_reserved_0(&self) -> bool {
        self.reserved() == 0
    }
}

impl ByteSlice for Ihw {
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { crate::any_as_u8_slice(self) }
    }
}

impl Debug for Ihw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.id();
        let reserved = self.reserved();
        let active_lanes = self.active_lanes();
        write!(f, "{:x} {:x} {:x}", id, reserved, active_lanes)
    }
}

impl PartialEq for Ihw {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.reserved == other.reserved
            && self.active_lanes == other.active_lanes
    }
}

#[repr(packed)]
pub struct Tdh {
    // 11:0 trigger_type
    // 12: internal_trigger, 13: no_data, 14: continuation, 15: reserved
    trigger_type_internal_trigger_no_data_continuation_reserved2: u16,
    trigger_bc_reserved1: u16, // 27:16 trigger_bc, 31:28 reserved,
    pub trigger_orbit: u32,    // 63:32
    // ID 0xe8
    reserved0_id: u16, // 71:64 reserved, 79:72 id
}
impl Tdh {
    fn reserved0(&self) -> u16 {
        self.reserved0_id & 0xFF
    }

    fn reserved1(&self) -> u16 {
        self.trigger_bc_reserved1 & 0xF000 // doesn't need shift as it should just be checked if equal to 0
    }

    fn trigger_bc(&self) -> u16 {
        self.trigger_bc_reserved1 & 0x0FFF
    }

    fn reserved2(&self) -> u16 {
        // 15th bit is reserved
        self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b1000_0000_0000_0000
    }

    fn continuation(&self) -> u16 {
        // 14th bit is continuation
        (self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b100_0000_0000_0000)
            >> 14
    }

    fn no_data(&self) -> u16 {
        // 13th bit is no_data
        (self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b10_0000_0000_0000)
            >> 13
    }

    fn internal_trigger(&self) -> u16 {
        // 12th bit is internal_trigger
        (self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b1_0000_0000_0000)
            >> 12
    }

    fn trigger_type(&self) -> u16 {
        // 11:0 is trigger_type
        self.trigger_type_internal_trigger_no_data_continuation_reserved2 & 0b1111_1111_1111
    }
}

impl StatusWord for Tdh {
    fn id(&self) -> u8 {
        (self.reserved0_id >> 8) as u8
    }

    fn print(&self) {
        let tmp_trigger_orbit = self.trigger_orbit;
        println!("TDH: reserved0: {:x}, trigger_orbit: {:x}, reserved1: {:x}, trigger_bc: {:x}, reserved2: {:x}, continuation: {:x}, no_data: {:x}, internal_trigger: {:x}, trigger_type: {:x}",
                 self.reserved0(),
                 tmp_trigger_orbit,
                 self.reserved1(),
                 self.trigger_bc(),
                 self.reserved2(),
                 self.continuation(),
                 self.no_data(),
                 self.internal_trigger(),
                 self.trigger_type());
    }

    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(Tdh {
            trigger_type_internal_trigger_no_data_continuation_reserved2: reader
                .read_u16::<LittleEndian>()
                .unwrap(),
            trigger_bc_reserved1: reader.read_u16::<LittleEndian>().unwrap(),
            trigger_orbit: reader.read_u32::<LittleEndian>().unwrap(),
            reserved0_id: reader.read_u16::<LittleEndian>().unwrap(),
        })
    }
    fn is_reserved_0(&self) -> bool {
        self.reserved0() == 0 && self.reserved1() == 0 && self.reserved2() == 0
    }
}

impl ByteSlice for Tdh {
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { crate::any_as_u8_slice(self) }
    }
}

impl Debug for Tdh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.id();
        let reserved0 = self.reserved0();
        let trigger_orbit = self.trigger_orbit;
        let reserved1 = self.reserved1();
        let trigger_bc = self.trigger_bc();
        let reserved2 = self.reserved2();
        let continuation = self.continuation();
        let no_data = self.no_data();
        let internal_trigger = self.internal_trigger();
        let trigger_type = self.trigger_type();
        write!(
            f,
            "{:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x}",
            id,
            reserved0,
            trigger_orbit,
            reserved1,
            trigger_bc,
            reserved2,
            continuation,
            no_data,
            internal_trigger,
            trigger_type
        )
    }
}

impl PartialEq for Tdh {
    fn eq(&self, other: &Self) -> bool {
        self.reserved0_id == other.reserved0_id
            && self.trigger_orbit == other.trigger_orbit
            && self.trigger_bc_reserved1 == other.trigger_bc_reserved1
            && self.trigger_type_internal_trigger_no_data_continuation_reserved2
                == other.trigger_type_internal_trigger_no_data_continuation_reserved2
    }
}

#[repr(packed)]
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
    pub fn reserved0(&self) -> u8 {
        self.res0_lane_starts_violation_res1_transmission_timeout_packet_done >> 4
    }
    pub fn lane_starts_violation(&self) -> bool {
        (self.res0_lane_starts_violation_res1_transmission_timeout_packet_done & 0b1000) != 0
    }
    pub fn reserved1(&self) -> u8 {
        self.res0_lane_starts_violation_res1_transmission_timeout_packet_done & 0b0100
    }
    pub fn transmission_timeout(&self) -> bool {
        (self.res0_lane_starts_violation_res1_transmission_timeout_packet_done & 0b0010) != 0
    }
    pub fn packet_done(&self) -> bool {
        (self.res0_lane_starts_violation_res1_transmission_timeout_packet_done & 0b0001) == 1
    }
    pub fn timeout_to_start(&self) -> bool {
        (self.timeout_to_start_timeout_start_stop_timeout_in_idle_res2 & 0b1000_0000) != 0
    }
    pub fn timeout_start_stop(&self) -> bool {
        (self.timeout_to_start_timeout_start_stop_timeout_in_idle_res2 & 0b0100_0000) != 0
    }
    pub fn timeout_in_idle(&self) -> bool {
        (self.timeout_to_start_timeout_start_stop_timeout_in_idle_res2 & 0b0010_0000) != 0
    }
    pub fn reserved2(&self) -> u8 {
        self.timeout_to_start_timeout_start_stop_timeout_in_idle_res2 & 0b0001_1111
    }
    pub fn lane_status_27_24(&self) -> u8 {
        self.lane_status_27_24
    }
    pub fn lane_status_23_16(&self) -> u16 {
        self.lane_status_23_16
    }
    pub fn lane_status_15_0(&self) -> u32 {
        self.lane_status_15_0
    }
}

impl StatusWord for Tdt {
    fn id(&self) -> u8 {
        self.id
    }

    fn print(&self) {
        println!(
            "TDT: {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x}",
            self.lane_starts_violation() as u8,
            self.transmission_timeout() as u8,
            self.packet_done() as u8,
            self.timeout_to_start() as u8,
            self.timeout_start_stop() as u8,
            self.timeout_in_idle() as u8,
            self.lane_status_27_24(),
            self.lane_status_23_16(),
            self.lane_status_15_0()
        );
    }

    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            lane_status_15_0: reader.read_u32::<LittleEndian>()?,
            lane_status_23_16: reader.read_u16::<LittleEndian>()?,
            lane_status_27_24: reader.read_u8()?,
            timeout_to_start_timeout_start_stop_timeout_in_idle_res2: reader.read_u8()?,
            res0_lane_starts_violation_res1_transmission_timeout_packet_done: reader.read_u8()?,
            id: reader.read_u8()?,
        })
    }
    fn is_reserved_0(&self) -> bool {
        self.reserved0() == 0 && self.reserved1() == 0 && self.reserved2() == 0
    }
}

impl ByteSlice for Tdt {
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { crate::any_as_u8_slice(self) }
    }
}

impl Debug for Tdt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.id();
        let lane_starts_violation = self.lane_starts_violation();
        let transmission_timeout = self.transmission_timeout();
        let packet_done = self.packet_done();
        let timeout_to_start = self.timeout_to_start();
        let timeout_start_stop = self.timeout_start_stop();
        let timeout_in_idle = self.timeout_in_idle();
        let lane_status_27_24 = self.lane_status_27_24();
        let lane_status_23_16 = self.lane_status_23_16();
        let lane_status_15_0 = self.lane_status_15_0();
        write!(
            f,
            "{:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x}",
            id,
            lane_starts_violation as u8,
            transmission_timeout as u8,
            packet_done as u8,
            timeout_to_start as u8,
            timeout_start_stop as u8,
            timeout_in_idle as u8,
            lane_status_27_24,
            lane_status_23_16,
            lane_status_15_0
        )
    }
}

impl PartialEq for Tdt {
    fn eq(&self, other: &Self) -> bool {
        self.res0_lane_starts_violation_res1_transmission_timeout_packet_done
            == other.res0_lane_starts_violation_res1_transmission_timeout_packet_done
            && self.timeout_to_start_timeout_start_stop_timeout_in_idle_res2
                == other.timeout_to_start_timeout_start_stop_timeout_in_idle_res2
            && self.lane_status_27_24 == other.lane_status_27_24
            && self.lane_status_23_16 == other.lane_status_23_16
            && self.lane_status_15_0 == other.lane_status_15_0
    }
}

pub struct Ddw0 {
    // 64:56 reserved0, 55:0 lane_status
    res3_lane_status: u64,
    // 71:68 index, 67: lane_starts_violation, 66: reserved0, 65: transmission_timeout, 64: reserved1
    index: u8,
    // ID: 0xe4
    id: u8, // 79:72
}

impl Ddw0 {
    pub fn index(&self) -> u8 {
        (self.index & 0xF0) >> 4
    }
    pub fn lane_starts_violation(&self) -> bool {
        (self.index & 0b1000) != 0
    }
    pub fn transmission_timeout(&self) -> bool {
        (self.index & 0b10) != 0
    }

    pub fn lane_status(&self) -> u64 {
        self.res3_lane_status & 0x00ff_ffff_ffff_ffff
    }
}

impl StatusWord for Ddw0 {
    fn id(&self) -> u8 {
        self.id
    }

    fn print(&self) {
        println!(
            "DDW0: {:x} {:x} {:x} {:x} {:x}",
            self.id,
            self.index(),
            self.lane_starts_violation() as u8,
            self.transmission_timeout() as u8,
            self.lane_status()
        );
    }

    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            res3_lane_status: reader.read_u64::<LittleEndian>()?,
            index: reader.read_u8()?,
            id: reader.read_u8()?,
        })
    }
    fn is_reserved_0(&self) -> bool {
        (self.index & 0b0000_0101) == 0 && (self.res3_lane_status & 0xFF00_0000_0000_0000) == 0
    }
}

impl ByteSlice for Ddw0 {
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { crate::any_as_u8_slice(self) }
    }
}

impl Debug for Ddw0 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.id();
        let index = self.index();
        let lane_starts_violation = self.lane_starts_violation();
        let transmission_timeout = self.transmission_timeout();
        let lane_status = self.lane_status();
        write!(
            f,
            "DDW0: {:x} {:x} {:x} {:x} {:x}",
            id, index, lane_starts_violation as u8, transmission_timeout as u8, lane_status
        )
    }
}

impl PartialEq for Ddw0 {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.index == other.index
            && self.res3_lane_status == other.res3_lane_status
    }
}

pub struct Cdw {
    calibration_word_index_lsb_calibration_user_fields: u64, // 63:48 calibration_word_index_LSB 47:0 calibration_user_fields
    calibration_word_index_msb: u8,                          // 71:64 calibration_word_index_MSB
    // ID: 0xF8
    pub id: u8,
}

impl Cdw {
    pub fn calibration_word_index(&self) -> u32 {
        ((self.calibration_word_index_msb as u32) << 16)
            | ((self.calibration_word_index_lsb_calibration_user_fields >> 48) as u32)
    }
    pub fn calibration_user_fields(&self) -> u64 {
        self.calibration_word_index_lsb_calibration_user_fields & 0xffff_ffff_ffff
    }
}

impl StatusWord for Cdw {
    fn id(&self) -> u8 {
        self.id
    }

    fn print(&self) {
        println!(
            "CDW: {:x} {:x} {:x}",
            self.id,
            self.calibration_word_index(),
            self.calibration_user_fields()
        );
    }

    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            calibration_word_index_lsb_calibration_user_fields: reader
                .read_u64::<LittleEndian>()?,
            calibration_word_index_msb: reader.read_u8()?,
            id: reader.read_u8()?,
        })
    }
    fn is_reserved_0(&self) -> bool {
        true // No reserved bits
    }
}

impl ByteSlice for Cdw {
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { crate::any_as_u8_slice(self) }
    }
}

impl Debug for Cdw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.id();
        let calibration_word_index = self.calibration_word_index();
        let calibration_user_fields = self.calibration_user_fields();
        write!(
            f,
            "CDW: {:x} {:x} {:x}",
            id, calibration_word_index, calibration_user_fields
        )
    }
}

impl PartialEq for Cdw {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.calibration_word_index_msb == other.calibration_word_index_msb
            && self.calibration_word_index_lsb_calibration_user_fields
                == other.calibration_word_index_lsb_calibration_user_fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ihw_read_write() {
        const VALID_ID: u8 = 0xE0;
        const ACTIVE_LANES_14_ACTIVE: u32 = 0x3F_FF;
        let raw_data_ihw = [0xFF, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0];
        if raw_data_ihw[9] != VALID_ID {
            panic!("Invalid ID");
        }
        let ihw = Ihw::load(&mut raw_data_ihw.as_slice()).unwrap();
        assert_eq!(ihw.id(), VALID_ID);
        assert!(ihw.is_reserved_0());
        assert_eq!(ihw.active_lanes(), ACTIVE_LANES_14_ACTIVE);
        ihw.print();
        let loaded_ihw = Ihw::load(&mut ihw.to_byte_slice()).unwrap();
        loaded_ihw.print();
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
        if raw_data_tdh[9] != VALID_ID {
            panic!("Invalid ID");
        }
        let tdh = Tdh::load(&mut raw_data_tdh.as_slice()).unwrap();
        tdh.print();
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
        assert!(raw_data_tdt[9] == VALID_ID);
        let tdt = Tdt::load(&mut raw_data_tdt.as_slice()).unwrap();
        tdt.print();
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
        tdt.print();
        assert_eq!(tdt.id(), VALID_ID);
        println!("tdt.is_reserved_0() = {}", tdt.is_reserved_0());
        println!(
            "{:x} {:x} {:x}",
            tdt.reserved0(),
            tdt.reserved1(),
            tdt.reserved2()
        );
        assert!(tdt.is_reserved_0());
        assert!(tdt.packet_done() == false);
        assert!(tdt.transmission_timeout());
        assert!(tdt.lane_starts_violation());
        assert!(tdt.timeout_to_start());
        assert!(tdt.timeout_start_stop());
        assert!(tdt.timeout_in_idle());
        assert!(tdt.lane_status_27_24() == LANE_24_AND_25_IN_ERROR);
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
    }

    #[test]
    fn ddw0_read_write() {
        const VALID_ID: u8 = 0xE4;
        let raw_data_ddw0 = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE4];
        assert!(raw_data_ddw0[9] == VALID_ID);
        let ddw0 = Ddw0::load(&mut raw_data_ddw0.as_slice()).unwrap();

        assert_eq!(ddw0.id(), VALID_ID);
        assert!(ddw0.is_reserved_0());
        assert!(ddw0.transmission_timeout() == false);
        assert!(ddw0.lane_starts_violation() == false);
        assert_eq!(ddw0.lane_status(), 0);
        let loaded_ddw0 = Ddw0::load(&mut ddw0.to_byte_slice()).unwrap();
        assert_eq!(ddw0, loaded_ddw0);
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
        assert!(raw_data_ddw0[9] == VALID_ID);
        let ddw0 = Ddw0::load(&mut raw_data_ddw0.as_slice()).unwrap();
        ddw0.print();
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
        println!("{:x}", combined_lane_status);
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
        assert!(cdw.id() == VALID_ID);
        assert!(cdw.is_reserved_0());
        assert_eq!(cdw.calibration_user_fields(), 0x050403020100);
        assert_eq!(cdw.calibration_word_index(), 0x080706);
        let loaded_cdw = Cdw::load(&mut cdw.to_byte_slice()).unwrap();
        assert_eq!(cdw, loaded_cdw);
    }
}
