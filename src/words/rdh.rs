use crate::ByteSlice;
// ITS data format: https://gitlab.cern.ch/alice-its-wp10-firmware/RU_mainFPGA/-/wikis/ITS%20Data%20Format#Introduction
use crate::GbtWord;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::fmt::{self, Debug, Display};

pub trait RDH: std::fmt::Debug + PartialEq + Sized + ByteSlice + Display + Sync + Send {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn load_from_rdh0<T: std::io::Read>(reader: &mut T, rdh0: Rdh0)
        -> Result<Self, std::io::Error>;
    fn get_link_id(&self) -> u8;
    /// Returns the size of the payload in bytes
    /// This is EXCLUDING the size of the RDH
    fn get_payload_size(&self) -> u16;
    fn get_offset_to_next(&self) -> u16;
}

// Newtype pattern used to enforce type safety on fields that are not byte-aligned
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(packed)]
pub(crate) struct CruidDw(pub(crate) u16); // 12 bit cru_id, 4 bit dw
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(packed)]
pub(crate) struct BcReserved(pub(crate) u32); // 12 bit bc, 20 bit reserved
#[repr(packed)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct DataformatReserved(pub(crate) u64); // 8 bit data_format, 56 bit reserved0
#[repr(packed)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct FeeId(pub(crate) u16); // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number

#[repr(packed)]
pub struct RdhCRUv7 {
    pub rdh0: Rdh0,
    pub offset_new_packet: u16,
    pub memory_size: u16,
    pub link_id: u8,
    pub packet_counter: u8,
    pub(crate) cruid_dw: CruidDw, // 12 bit cru_id, 4 bit dw
    pub rdh1: Rdh1,
    pub(crate) dataformat_reserved0: DataformatReserved, // 8 bit data_format, 56 bit reserved0
    pub rdh2: Rdh2,
    pub reserved1: u64,
    pub rdh3: Rdh3,
    pub reserved2: u64,
}
impl RdhCRUv7 {
    pub fn cru_id(&self) -> u16 {
        // Get the cru_id present in the 12 LSB
        self.cruid_dw.0 & 0x0FFF
    }
    pub fn dw(&self) -> u8 {
        // Get the dw present in the 4 MSB
        ((self.cruid_dw.0 & 0xF000) >> 12) as u8
    }
    pub fn data_format(&self) -> u8 {
        // Get the data_format present in the 8 LSB
        (self.dataformat_reserved0.0 & 0x00000000000000FF) as u8
    }
    pub fn reserved0(&self) -> u64 {
        // Get the reserved0 present in the 56 MSB
        (self.dataformat_reserved0.0 & 0xFFFFFFFFFFFFFF00) >> 8
    }
}

impl Display for RdhCRUv7 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let header_text_top = "RDH   Header  FEE   Sys   Offset  Link  Packet    BC   Orbit       Data       Trigger   Pages    Stop";
        let header_text_bottom = "ver   size    ID    ID    next    ID    counter        counter     format     type      counter  bit";
        //Needed?  Memory   CRU   DW");
        //Needed?    size     ID    ID\n");
        let tmp_offset = self.offset_new_packet;
        let tmp_link = self.link_id;
        let tmp_packet_cnt = self.packet_counter;
        let rdhcru_fields0 = format!("{tmp_offset:<8}{tmp_link:<6}{tmp_packet_cnt:<10}");
        write!(
            f,
            "{header_text_top}\n       {header_text_bottom}\n       {}{}{}{:<11}{}",
            self.rdh0,
            rdhcru_fields0,
            self.rdh1,
            self.data_format(),
            self.rdh2
        )
    }
}

impl RDH for RdhCRUv7 {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<RdhCRUv7, std::io::Error> {
        let rdh0 = match Rdh0::load(reader) {
            Ok(rdh0) => rdh0,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Err(std::io::ErrorKind::UnexpectedEof.into())
            }
            Err(e) => return Err(e),
        };
        Self::load_from_rdh0(reader, rdh0)
    }

    fn load_from_rdh0<T: std::io::Read>(
        reader: &mut T,
        rdh0: Rdh0,
    ) -> Result<Self, std::io::Error> {
        let offset_new_packet = reader.read_u16::<LittleEndian>().unwrap();
        let memory_size = reader.read_u16::<LittleEndian>().unwrap();
        let link_id = reader.read_u8().unwrap();
        let packet_counter = reader.read_u8().unwrap();
        // cru_id is 12 bit and the following dw is 4 bit
        let tmp_cruid_dw = CruidDw(reader.read_u16::<LittleEndian>().unwrap());
        let rdh1 = Rdh1::load(reader).expect("Error while loading Rdh1");
        // Now the next 64 bits contain the reserved0 and data_format
        // [7:0]data_format, [63:8]reserved0
        let tmp_dataformat_reserverd0 =
            DataformatReserved(reader.read_u64::<LittleEndian>().unwrap());
        let rdh2 = Rdh2::load(reader).expect("Error while loading Rdh2");
        let reserved1 = reader.read_u64::<LittleEndian>().unwrap();
        let rdh3 = Rdh3::load(reader).expect("Error while loading Rdh3");
        let reserved2 = reader.read_u64::<LittleEndian>().unwrap();
        // Finally return the RdhCRU
        Ok(RdhCRUv7 {
            rdh0,
            offset_new_packet,
            memory_size,
            link_id,
            packet_counter,
            cruid_dw: tmp_cruid_dw,
            rdh1,
            dataformat_reserved0: tmp_dataformat_reserverd0,
            rdh2,
            reserved1,
            rdh3,
            reserved2,
        })
    }

    fn get_link_id(&self) -> u8 {
        self.link_id
    }
    fn get_payload_size(&self) -> u16 {
        self.memory_size - 64 // 64 bytes are the RDH size. Payload size is the memory size minus the RDH size.
    }
    fn get_offset_to_next(&self) -> u16 {
        self.offset_new_packet
    }
}
impl ByteSlice for RdhCRUv7 {
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { crate::any_as_u8_slice(self) }
    }
}

impl Debug for RdhCRUv7 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let tmp_rdh0 = &self.rdh0;
        let tmp_offset_new_packet = self.offset_new_packet;
        let tmp_memory_size = self.memory_size;
        let tmp_link_id = self.link_id;
        let tmp_packet_counter = self.packet_counter;
        let tmp_cruid = &self.cru_id();
        let tmp_dw = &self.dw();
        let tmp_rdh1 = &self.rdh1;
        let tmp_data_format = self.data_format();
        let tmp_reserved0 = self.reserved0();
        let tmp_rdh2 = &self.rdh2;
        let tmp_reserved1 = self.reserved1;
        let tmp_rdh3 = &self.rdh3;
        let tmp_reserved2 = self.reserved2;

        write!(f, "RdhCRUv7: rdh0: {tmp_rdh0:?}, offset_new_packet: {tmp_offset_new_packet:x?}, memory_size: {tmp_memory_size:x?}, link_id: {tmp_link_id:x?}, packet_counter: {tmp_packet_counter:x?}, cruid: {tmp_cruid:x?}, dw: {tmp_dw:x?}, rdh1: {tmp_rdh1:?}, data_format: {tmp_data_format:x?}, reserved0: {tmp_reserved0:x?}, rdh2: {tmp_rdh2:?}, reserved1: {tmp_reserved1:x?}, rdh3: {tmp_rdh3:?}, reserved2: {tmp_reserved2:x?}")
    }
}

impl PartialEq for RdhCRUv7 {
    fn eq(&self, other: &Self) -> bool {
        self.rdh0 == other.rdh0
            && self.offset_new_packet == other.offset_new_packet
            && self.memory_size == other.memory_size
            && self.link_id == other.link_id
            && self.packet_counter == other.packet_counter
            && self.cruid_dw == other.cruid_dw
            && self.rdh1 == other.rdh1
            && self.dataformat_reserved0 == other.dataformat_reserved0
            && self.rdh2 == other.rdh2
            && self.reserved1 == other.reserved1
            && self.rdh3 == other.rdh3
            && self.reserved2 == other.reserved2
    }
}

#[repr(packed)]
pub struct RdhCRUv6 {
    pub rdh0: Rdh0,
    pub offset_new_packet: u16,
    pub memory_size: u16,
    pub link_id: u8,
    pub packet_counter: u8,
    pub(crate) cruid_dw: CruidDw, // 12 bit cru_id, 4 bit dw
    pub rdh1: Rdh1,
    pub reserved0: u64,
    pub rdh2: Rdh2,
    pub reserved1: u64,
    pub rdh3: Rdh3,
    pub reserved2: u64,
}
impl RdhCRUv6 {
    pub fn cru_id(&self) -> u16 {
        // Get the cru_id present in the 12 LSB
        self.cruid_dw.0 & 0x0FFF
    }
    pub fn dw(&self) -> u8 {
        // Get the dw present in the 4 MSB
        ((self.cruid_dw.0 & 0xF000) >> 12) as u8
    }
}

impl Display for RdhCRUv6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let header_text_top    = "RDH   Header  FEE   Sys   Offset  Link  Packet    BC   Orbit       Trigger   Pages    Stop";
        let header_text_bottom = "ver   size    ID    ID    next    ID    counter        counter     type      counter  bit";
        //Needed?  Memory   CRU   DW");
        //Needed?    size     ID    ID\n");
        let tmp_offset = self.offset_new_packet;
        let tmp_link = self.link_id;
        let tmp_packet_cnt = self.packet_counter;
        let rdhcru_fields0 = format!("{tmp_offset:<8}{tmp_link:<6}{tmp_packet_cnt:<10}");
        write!(
            f,
            "{header_text_top}\n       {header_text_bottom}\n       {}{}{}{}",
            self.rdh0, rdhcru_fields0, self.rdh1, self.rdh2
        )
    }
}

impl RDH for RdhCRUv6 {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<RdhCRUv6, std::io::Error> {
        let rdh0 = match Rdh0::load(reader) {
            Ok(rdh0) => rdh0,
            Err(e) => return Err(e),
        };
        Self::load_from_rdh0(reader, rdh0)
    }
    fn load_from_rdh0<T: std::io::Read>(
        reader: &mut T,
        rdh0: Rdh0,
    ) -> Result<Self, std::io::Error> {
        let offset_new_packet = reader.read_u16::<LittleEndian>().unwrap();
        let memory_size = reader.read_u16::<LittleEndian>().unwrap();
        let link_id = reader.read_u8().unwrap();
        let packet_counter = reader.read_u8().unwrap();
        // cru_id is 12 bit and the following dw is 4 bit
        let tmp_cruid_dw = CruidDw(reader.read_u16::<LittleEndian>().unwrap());
        let rdh1 = Rdh1::load(reader).expect("Error while loading Rdh1");
        let reserved0 = reader.read_u64::<LittleEndian>().unwrap();
        let rdh2 = Rdh2::load(reader).expect("Error while loading Rdh2");
        let reserved1 = reader.read_u64::<LittleEndian>().unwrap();
        let rdh3 = Rdh3::load(reader).expect("Error while loading Rdh3");
        let reserved2 = reader.read_u64::<LittleEndian>().unwrap();
        // Finally return the RdhCRU
        Ok(RdhCRUv6 {
            rdh0,
            offset_new_packet,
            memory_size,
            link_id,
            packet_counter,
            cruid_dw: tmp_cruid_dw,
            rdh1,
            reserved0,
            rdh2,
            reserved1,
            rdh3,
            reserved2,
        })
    }
    fn get_link_id(&self) -> u8 {
        self.link_id
    }
    fn get_payload_size(&self) -> u16 {
        self.memory_size - 64 // 64 bytes are the RDH size. Payload size is the memory size minus the RDH size.
    }
    fn get_offset_to_next(&self) -> u16 {
        self.offset_new_packet
    }
}

impl ByteSlice for RdhCRUv6 {
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { crate::any_as_u8_slice(self) }
    }
}

impl Debug for RdhCRUv6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_rdh0 = &self.rdh0;
        let tmp_offset_new_packet = self.offset_new_packet;
        let tmp_memory_size = self.memory_size;
        let tmp_link_id = self.link_id;
        let tmp_packet_counter = self.packet_counter;
        let tmp_cruid = &self.cru_id();
        let tmp_dw = &self.dw();
        let tmp_rdh1 = &self.rdh1;
        let tmp_reserved0 = self.reserved0;
        let tmp_rdh2 = &self.rdh2;
        let tmp_reserved1 = self.reserved1;
        let tmp_rdh3 = &self.rdh3;
        let tmp_reserved2 = self.reserved2;

        write!(f, "RdhCRUv7: rdh0: {tmp_rdh0:?}, offset_new_packet: {tmp_offset_new_packet:x?}, memory_size: {tmp_memory_size:x?}, link_id: {tmp_link_id:x?}, packet_counter: {tmp_packet_counter:x?}, cruid: {tmp_cruid:x?}, dw: {tmp_dw:x?}, rdh1: {tmp_rdh1:?}, reserved0: {tmp_reserved0:x?}, rdh2: {tmp_rdh2:?}, reserved1: {tmp_reserved1:x?}, rdh3: {tmp_rdh3:?}, reserved2: {tmp_reserved2:x?}")
    }
}

impl PartialEq for RdhCRUv6 {
    fn eq(&self, other: &RdhCRUv6) -> bool {
        self.rdh0 == other.rdh0
            && self.offset_new_packet == other.offset_new_packet
            && self.memory_size == other.memory_size
            && self.link_id == other.link_id
            && self.packet_counter == other.packet_counter
            && self.cruid_dw == other.cruid_dw
            && self.rdh1 == other.rdh1
            && self.reserved0 == other.reserved0
            && self.rdh2 == other.rdh2
            && self.reserved1 == other.reserved1
            && self.rdh3 == other.rdh3
            && self.reserved2 == other.reserved2
    }
}

#[repr(packed)]
pub struct Rdh0 {
    // Represents 64 bit
    pub header_id: u8,
    pub header_size: u8,
    pub(crate) fee_id: FeeId, // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number
    pub priority_bit: u8,
    pub system_id: u8,
    pub reserved0: u16,
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

impl GbtWord for Rdh0 {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Rdh0, std::io::Error> {
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
        Ok(Rdh0 {
            header_id: load_bytes!(1)[0],
            header_size: load_bytes!(1)[0],
            fee_id: FeeId(LittleEndian::read_u16(&load_bytes!(2))),
            priority_bit: load_bytes!(1)[0],
            system_id: load_bytes!(1)[0],
            reserved0: LittleEndian::read_u16(&load_bytes!(2)),
        })
    }
}
impl Debug for Rdh0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_header_id = self.header_id;
        let tmp_header_size = self.header_size;
        let tmp_fee_id = self.fee_id;
        let tmp_priority_bit = self.priority_bit;
        let tmp_system_id = self.system_id;
        let tmp_reserved0 = self.reserved0;

        write!(f, "Rdh0: header_id: {tmp_header_id:x?}, header_size: {tmp_header_size:x?}, fee_id: {tmp_fee_id:x?}, priority_bit: {tmp_priority_bit:x?}, system_id: {tmp_system_id:x?}, reserved0: {tmp_reserved0:x?}")
    }
}
impl PartialEq for Rdh0 {
    fn eq(&self, other: &Self) -> bool {
        self.header_id == other.header_id
            && self.header_size == other.header_size
            && self.fee_id == other.fee_id
            && self.priority_bit == other.priority_bit
            && self.system_id == other.system_id
            && self.reserved0 == other.reserved0
    }
}

#[repr(packed)]
pub struct Rdh1 {
    // Rdh1 is 64 bit total
    pub(crate) bc_reserved0: BcReserved, //bunch counter 12 bit + reserved 20 bit
    pub orbit: u32,                      // 32 bit
}

impl Rdh1 {
    // only meant for unit tests
    pub const fn test_new(bc: u16, orbit: u32, reserved0: u32) -> Self {
        Rdh1 {
            bc_reserved0: BcReserved((bc as u32) | (reserved0 << 12)),
            orbit,
        }
    }

    pub fn bc(&self) -> u16 {
        (self.bc_reserved0.0 & 0x0FFF) as u16
    }
    pub fn reserved0(&self) -> u32 {
        self.bc_reserved0.0 >> 12
    }
}

impl GbtWord for Rdh1 {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Rdh1, std::io::Error> {
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

        Ok(Rdh1 {
            bc_reserved0: BcReserved(LittleEndian::read_u32(&load_bytes!(4))),
            orbit: LittleEndian::read_u32(&load_bytes!(4)),
        })
    }
}
impl Debug for Rdh1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_bc = self.bc();
        let tmp_reserved0 = self.reserved0();
        let tmp_orbit = self.orbit;
        write!(
            f,
            "Rdh1: bc: {tmp_bc:x?}, reserved0: {tmp_reserved0:x?}, orbit: {tmp_orbit:x?}"
        )
    }
}
impl PartialEq for Rdh1 {
    fn eq(&self, other: &Self) -> bool {
        self.bc_reserved0 == other.bc_reserved0 && self.orbit == other.orbit
    }
}

impl Display for Rdh1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_orbit = self.orbit;
        let orbit_as_hex = format!("{tmp_orbit:#x}");
        write!(f, "{:<5}{:<12}", self.bc(), orbit_as_hex)
    }
}

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct Rdh2 {
    pub trigger_type: u32, // 32 bit
    pub pages_counter: u16,
    pub stop_bit: u8,
    pub reserved0: u8,
}

impl GbtWord for Rdh2 {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Rdh2, std::io::Error> {
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

        Ok(Rdh2 {
            trigger_type: LittleEndian::read_u32(&load_bytes!(4)),
            pages_counter: LittleEndian::read_u16(&load_bytes!(2)),
            stop_bit: load_bytes!(1)[0],
            reserved0: load_bytes!(1)[0],
        })
    }
}
impl Debug for Rdh2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_trigger_type = self.trigger_type;
        let tmp_pages_counter = self.pages_counter;
        let tmp_stop_bit = self.stop_bit;
        let tmp_reserved0 = self.reserved0;
        write!(
            f,
            "Rdh2: trigger_type: {tmp_trigger_type:x?}, pages_counter: {tmp_pages_counter:x?}, stop_bit: {tmp_stop_bit:x?}, reserved0: {tmp_reserved0:x?}"
        )
    }
}

impl PartialEq for Rdh2 {
    fn eq(&self, other: &Self) -> bool {
        self.trigger_type == other.trigger_type
            && self.pages_counter == other.pages_counter
            && self.stop_bit == other.stop_bit
            && self.reserved0 == other.reserved0
    }
}

impl Display for Rdh2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_trigger_type = self.trigger_type;
        let tmp_pages_counter = self.pages_counter;
        let trigger_type_as_hex = format!("{tmp_trigger_type:#x}");
        write!(
            f,
            "{:<10}{:<9}{:<5}",
            trigger_type_as_hex, tmp_pages_counter, self.stop_bit
        )
    }
}

#[repr(packed)]
pub struct Rdh3 {
    pub detector_field: u32, // 23:4 is reserved
    pub par_bit: u16,
    pub reserved0: u16,
}
impl GbtWord for Rdh3 {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Rdh3, std::io::Error> {
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

        Ok(Rdh3 {
            detector_field: LittleEndian::read_u32(&load_bytes!(4)),
            par_bit: LittleEndian::read_u16(&load_bytes!(2)),
            reserved0: LittleEndian::read_u16(&load_bytes!(2)),
        })
    }
}
impl PartialEq for Rdh3 {
    fn eq(&self, other: &Self) -> bool {
        self.detector_field == other.detector_field
            && self.par_bit == other.par_bit
            && self.reserved0 == other.reserved0
    }
}
impl Debug for Rdh3 {
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
    use std::fs::{self, File, OpenOptions};
    use std::io::Write;
    use std::{io::BufReader, path::PathBuf};

    // Verifies that the RdhCruv7 struct is serialized and deserialized correctly
    #[test]
    #[ignore] // Large test ignored in normal cases, useful for debugging
    fn test_load_rdhcruv7() {
        // Create an instace of an RDH-CRU v7
        // write it to a file
        // read it back
        // assert that they are equal
        let filename = "test_files/test_rdhcruv7.raw";
        let correct_rdh_cru = RdhCRUv7 {
            rdh0: Rdh0 {
                header_id: 0x7,
                header_size: 0x40,
                fee_id: FeeId(0x502A),
                priority_bit: 0x0,
                system_id: 0x20,
                reserved0: 0,
            },
            offset_new_packet: 0x13E0,
            memory_size: 0x13E0,
            link_id: 0x0,
            packet_counter: 0x0,
            cruid_dw: CruidDw(0x0018),
            rdh1: Rdh1 {
                bc_reserved0: BcReserved(0x0),
                orbit: 0x0b7dd575,
            },
            dataformat_reserved0: DataformatReserved(0x2),
            rdh2: Rdh2 {
                trigger_type: 0x00006a03,
                pages_counter: 0x0,
                stop_bit: 0x0,
                reserved0: 0x0,
            },
            reserved1: 0x0,
            rdh3: Rdh3 {
                detector_field: 0x0,
                par_bit: 0x0,
                reserved0: 0x0,
            },
            reserved2: 0x0,
        };
        let filepath = PathBuf::from(filename);
        let mut file = File::create(&filepath).unwrap();
        // Write the RDH-CRU v7 to the file
        let serialized_rdh = correct_rdh_cru.to_byte_slice();
        file.write_all(serialized_rdh).unwrap();
        //correct_rdh_cru.write(&mut file).unwrap();
        // Open and read with the manual serialization method

        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(&filepath)
            .expect("File not found");
        let mut buf_reader = BufReader::new(file);
        let rdh_cru = RdhCRUv7::load(&mut buf_reader).expect("Failed to load RdhCRUv7");
        //let rdh_cru_binrw = RdhCRUv7::read(&mut buf_reader).expect("Failed to load RdhCRUv7");
        // Assert that the two methods are equal
        assert_eq!(rdh_cru, correct_rdh_cru);
        //assert_eq!(rdh_cru_binrw, correct_rdh_cru);
        // This function currently corresponds to the unlink function on Unix and the DeleteFile function on Windows. Note that, this may change in the future.
        // More info: https://doc.rust-lang.org/std/fs/fn.remove_file.html
        fs::remove_file(&filepath).unwrap();
    }

    // Verifies that the RdhCruv6 struct is serialized and deserialized correctly
    #[test]
    #[ignore] // Large test ignored in normal cases, useful for debugging
    fn test_load_rdhcruv6() {
        // Create an instace of an RDH-CRU v6
        // write it to a file
        // read it back
        // assert that they are equal
        let filename = "test_files/test_rdhcruv6.raw";
        let correct_rdhcruv6 = RdhCRUv6 {
            rdh0: Rdh0 {
                header_id: 0x6,
                header_size: 0x40,
                fee_id: FeeId(0x502A),
                priority_bit: 0x0,
                system_id: 0x20,
                reserved0: 0,
            },
            offset_new_packet: 0x13E0,
            memory_size: 0x13E0,
            link_id: 0x2,
            packet_counter: 0x1,
            cruid_dw: CruidDw(0x0018),
            rdh1: Rdh1 {
                bc_reserved0: BcReserved(0x0),
                orbit: 0x0b7dd575,
            },
            reserved0: 0x0,
            rdh2: Rdh2 {
                trigger_type: 0x00006a03,
                pages_counter: 0x0,
                stop_bit: 0x0,
                reserved0: 0x0,
            },
            reserved1: 0x0,
            rdh3: Rdh3 {
                detector_field: 0x0,
                par_bit: 0x0,
                reserved0: 0x0,
            },
            reserved2: 0x0,
        };

        let filepath = PathBuf::from(filename);
        let mut file = File::create(&filepath).unwrap();
        // Write the RDH-CRU v6 to the file
        let serialized_rdh = correct_rdhcruv6.to_byte_slice();
        file.write_all(serialized_rdh).unwrap();
        //correct_rdhcruv6.write(&mut file).unwrap();

        // Open, and read the file manual way
        let file = OpenOptions::new().read(true).open(&filepath).unwrap();
        let mut buf_reader = BufReader::new(file);
        let rdh_cru = RdhCRUv6::load(&mut buf_reader).expect("Failed to load RdhCRUv6");
        // Check that both the manual and binrw way of reading the file are correct
        assert_eq!(rdh_cru, correct_rdhcruv6);

        // This function currently corresponds to the unlink function on Unix and the DeleteFile function on Windows. Note that, this may change in the future.
        // More info: https://doc.rust-lang.org/std/fs/fn.remove_file.html
        fs::remove_file(&filepath).unwrap();
    }

    #[test]
    fn test_load_rdhcruv7_from_byte_slice() {
        // Create an instace of an RDH-CRU v7
        // byte slice values taken from a valid rdh from real data
        let rdhcruv7 = RdhCRUv7::load(
            &mut &[
                0x07, 0x40, 0x2a, 0x50, 0x00, 0x20, 0x00, 0x00, 0xe0, 0x13, 0xe0, 0x13, 0x00, 0x00,
                0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x75, 0xd5, 0x7d, 0x0b, 0x02, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x03, 0x6a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ][..],
        )
        .unwrap();
        // Check that the fields are correct
        println!("{rdhcruv7:#?}");

        let rdh_from_old = RdhCRUv7::load(&mut &rdhcruv7.to_byte_slice()[..]).unwrap();
        println!("{rdh_from_old}");
        assert_eq!(rdhcruv7, rdh_from_old);
    }
}
//
