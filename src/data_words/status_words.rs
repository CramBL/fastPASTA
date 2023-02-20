use std::fmt::Debug;

use byteorder::{ByteOrder, LittleEndian};

use crate::{pretty_print_hex_field, pretty_print_name_hex_fields, ByteSlice};

pub trait StatusWord: std::fmt::Debug + PartialEq + Sized + ByteSlice {
    fn load_from_id<T: std::io::Read>(id: u8, reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn id(&self) -> u8;
    fn print(&self);
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
}

#[repr(packed)]
pub struct Ihw {
    // Total of 80 bits
    id: u16,           // 79:72
    reserved: u32,     // 71:28
    active_lanes: u32, // 27:0
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
    fn load_from_id<T: std::io::Read>(id: u8, reader: &mut T) -> Result<Self, std::io::Error> {
        // Create a helper macro for loading an array of the given size from
        // the reader.
        macro_rules! load_bytes {
            ($size:literal) => {{
                // Create a buffer array of the given size
                let mut buf = [0u8; $size];
                // Read into the buffer
                reader.read_exact(&mut buf)?;
                buf
            }};
        }
        // Read the next byte, OR with the id shifted to the left by 8 bits
        let id: u16 = load_bytes!(1)[0] as u16 | ((id as u16) << 8);
        Ok(Ihw {
            id,
            reserved: LittleEndian::read_u32(&load_bytes!(4)),
            active_lanes: LittleEndian::read_u32(&load_bytes!(4)),
        })
    }
    fn id(&self) -> u8 {
        (self.id >> 8) as u8
    }
    fn print(&self) {
        pretty_print_name_hex_fields!(Ihw, self, id, reserved, active_lanes);
    }

    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        // Create a helper macro for loading an array of the given size from
        // the reader.
        macro_rules! load_bytes {
            ($size:literal) => {{
                // Create a buffer array of the given size
                let mut buf = [0u8; $size];
                // Read into the buffer
                reader.read_exact(&mut buf)?;
                buf
            }};
        }
        Ok(Ihw {
            id: LittleEndian::read_u16(&load_bytes!(2)),
            reserved: LittleEndian::read_u32(&load_bytes!(4)),
            active_lanes: LittleEndian::read_u32(&load_bytes!(4)),
        })
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
    // ID 0xe8
    id_reserved0: u16,         // 79:72, 71:64
    pub trigger_orbit: u32,    // 63:32
    reserved1_trigger_bc: u16, // 31:28 reserved, 27:16 trigger_bc
    // 15: reserved, 14: continuation, 13: no_data, 12: internal_trigger, 11:0 trigger_type
    reserved2_continuation_no_data_internal_trigger_trigger_type: u16,
}
impl Tdh {
    fn reserved0(&self) -> u16 {
        self.id_reserved0 & 0xFF
    }

    fn reserved1(&self) -> u16 {
        self.reserved1_trigger_bc & 0xF000 // doesn't need shift as it should just be checked if equal to 0
    }

    fn trigger_bc(&self) -> u16 {
        self.reserved1_trigger_bc & 0x0FFF
    }

    fn reserved2(&self) -> u16 {
        // 15th bit is reserved
        self.reserved2_continuation_no_data_internal_trigger_trigger_type & 0b1000_0000_0000_0000
    }
    fn continuation(&self) -> u16 {
        // 14th bit is continuation
        self.reserved2_continuation_no_data_internal_trigger_trigger_type
            & 0b100_0000_0000_0000 >> 14
    }

    fn no_data(&self) -> u16 {
        // 13th bit is no_data
        self.reserved2_continuation_no_data_internal_trigger_trigger_type
            & 0b10_0000_0000_0000 >> 13
    }

    fn internal_trigger(&self) -> u16 {
        // 12th bit is internal_trigger
        self.reserved2_continuation_no_data_internal_trigger_trigger_type & 0b1_0000_0000_0000 >> 12
    }

    fn trigger_type(&self) -> u16 {
        // 11:0 is trigger_type
        self.reserved2_continuation_no_data_internal_trigger_trigger_type & 0b1111_1111_1111
    }
}

impl StatusWord for Tdh {
    fn load_from_id<T: std::io::Read>(id: u8, reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        macro_rules! load_bytes {
            ($size:literal) => {{
                // Create a buffer array of the given size
                let mut buf = [0u8; $size];
                // Read into the buffer
                reader.read_exact(&mut buf)?;
                buf
            }};
        }
        // Read the next byte, OR with the id shifted to the left by 8 bits
        let id: u16 = load_bytes!(1)[0] as u16 | ((id as u16) << 8);
        Ok(Tdh {
            id_reserved0: id,
            trigger_orbit: LittleEndian::read_u32(&load_bytes!(4)),
            reserved1_trigger_bc: LittleEndian::read_u16(&load_bytes!(2)),
            reserved2_continuation_no_data_internal_trigger_trigger_type: LittleEndian::read_u16(
                &load_bytes!(2),
            ),
        })
    }

    fn id(&self) -> u8 {
        (self.id_reserved0 >> 8) as u8
    }

    fn print(&self) {
        let tmp_trigger_orbit = self.trigger_orbit;
        println!("TDH: reserved0: {}, trigger_orbit: {}, reserved1: {}, trigger_bc: {}, reserved2: {}, continuation: {}, no_data: {}, internal_trigger: {}, trigger_type: {}",
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
        macro_rules! load_bytes {
            ($size:literal) => {{
                // Create a buffer array of the given size
                let mut buf = [0u8; $size];
                // Read into the buffer
                reader.read_exact(&mut buf)?;
                buf
            }};
        }
        Ok(Tdh {
            id_reserved0: LittleEndian::read_u16(&load_bytes!(2)),
            trigger_orbit: LittleEndian::read_u32(&load_bytes!(4)),
            reserved1_trigger_bc: LittleEndian::read_u16(&load_bytes!(2)),
            reserved2_continuation_no_data_internal_trigger_trigger_type: LittleEndian::read_u16(
                &load_bytes!(2),
            ),
        })
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
        self.id_reserved0 == other.id_reserved0
            && self.trigger_orbit == other.trigger_orbit
            && self.reserved1_trigger_bc == other.reserved1_trigger_bc
            && self.reserved2_continuation_no_data_internal_trigger_trigger_type
                == other.reserved2_continuation_no_data_internal_trigger_trigger_type
    }
}

#[repr(packed)]
pub struct Tdt {
    // ID 0xf0
    id: u8,
    // 71:68 reserved, 67: lane_starts_violation, 66: reserved, 65: transmission_timeout, 64: packet_done
    res0_lane_starts_violation_res1_transmission_timeout_packet_done: u8,
    // 63: timeout_to_start, 62: timeout_start_stop, 61: timeout_in_idle, 60:56 Reserved
    timeout_to_start_timeout_start_stop_timeout_in_idle_res2: u8,
    // 55:00 lane_status
    lane_status_27_24: u8,
    lane_status_23_16: u16,
    lane_status_15_0: u32,
}

impl Tdt {
    pub fn reserved0(&self) -> u8 {
        self.res0_lane_starts_violation_res1_transmission_timeout_packet_done >> 4
    }
    pub fn lane_starts_violation(&self) -> bool {
        self.res0_lane_starts_violation_res1_transmission_timeout_packet_done & 0b1000 != 0
    }
    pub fn reserved1(&self) -> u8 {
        self.res0_lane_starts_violation_res1_transmission_timeout_packet_done & 0b0100
    }
    pub fn transmission_timeout(&self) -> bool {
        self.res0_lane_starts_violation_res1_transmission_timeout_packet_done & 0b0010 != 0
    }
    pub fn packet_done(&self) -> bool {
        self.res0_lane_starts_violation_res1_transmission_timeout_packet_done & 0b0001 == 1
    }
    pub fn timeout_to_start(&self) -> bool {
        self.timeout_to_start_timeout_start_stop_timeout_in_idle_res2 & 0b1000_0000 != 0
    }
    pub fn timeout_start_stop(&self) -> bool {
        self.timeout_to_start_timeout_start_stop_timeout_in_idle_res2 & 0b0100_0000 != 0
    }
    pub fn timeout_in_idle(&self) -> bool {
        self.timeout_to_start_timeout_start_stop_timeout_in_idle_res2 & 0b0010_0000 != 0
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
    fn load_from_id<T: std::io::Read>(id: u8, reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        macro_rules! load_bytes {
            ($size:literal) => {{
                // Create a buffer array of the given size
                let mut buf = [0u8; $size];
                // Read into the buffer
                reader.read_exact(&mut buf)?;
                buf
            }};
        }
        Ok(Tdt {
            id,
            res0_lane_starts_violation_res1_transmission_timeout_packet_done: load_bytes!(1)[0],
            timeout_to_start_timeout_start_stop_timeout_in_idle_res2: load_bytes!(1)[0],
            lane_status_27_24: load_bytes!(1)[0],
            lane_status_23_16: LittleEndian::read_u16(&load_bytes!(2)),
            lane_status_15_0: LittleEndian::read_u32(&load_bytes!(4)),
        })
    }

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
        macro_rules! load_bytes {
            ($size:literal) => {{
                // Create a buffer array of the given size
                let mut buf = [0u8; $size];
                // Read into the buffer
                reader.read_exact(&mut buf)?;
                buf
            }};
        }
        Ok(Tdt {
            id: load_bytes!(1)[0],
            res0_lane_starts_violation_res1_transmission_timeout_packet_done: load_bytes!(1)[0],
            timeout_to_start_timeout_start_stop_timeout_in_idle_res2: load_bytes!(1)[0],
            lane_status_27_24: load_bytes!(1)[0],
            lane_status_23_16: LittleEndian::read_u16(&load_bytes!(2)),
            lane_status_15_0: LittleEndian::read_u32(&load_bytes!(4)),
        })
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
    // ID: 0xe4
    id: u8, // 79:72
    // 71:68 index, 67: lane_starts_violation, 66: reserved0, 65: transmission_timeout, 64: reserved1
    index: u8,
    // 64:56 reserved0, 55:0 lane_status
    res3_lane_status: u64,
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

    pub fn is_reserved_0(&self) -> bool {
        (self.index & 0b0000_0101) == 0 && (self.res3_lane_status & 0xFF00_0000_0000_0000) == 0
    }

    pub fn lane_status(&self) -> u64 {
        self.res3_lane_status & 0x00ff_ffff_ffff_ffff
    }
}

impl StatusWord for Ddw0 {
    fn load_from_id<T: std::io::Read>(id: u8, reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        macro_rules! load_bytes {
            ($size:literal) => {{
                // Create a buffer array of the given size
                let mut buf = [0u8; $size];
                // Read into the buffer
                reader.read_exact(&mut buf)?;
                buf
            }};
        }
        Ok(Ddw0 {
            id,
            index: load_bytes!(1)[0],
            res3_lane_status: LittleEndian::read_u64(&load_bytes!(8)),
        })
    }

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
        macro_rules! load_bytes {
            ($size:literal) => {{
                // Create a buffer array of the given size
                let mut buf = [0u8; $size];
                // Read into the buffer
                reader.read_exact(&mut buf)?;
                buf
            }};
        }
        Ok(Ddw0 {
            id: load_bytes!(1)[0],
            index: load_bytes!(1)[0],
            res3_lane_status: LittleEndian::read_u64(&load_bytes!(8)),
        })
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
    // ID: 0xF8
    id: u8, // 79:72
            // 71:48 calibration_word_index
            // 47:0 calibration_user_fields
}
