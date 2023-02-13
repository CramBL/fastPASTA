use crate::ByteSlice;
// ITS data format: https://gitlab.cern.ch/alice-its-wp10-firmware/RU_mainFPGA/-/wikis/ITS%20Data%20Format#Introduction
use crate::validators::rdh::GbtError;
use crate::{
    pretty_print_hex_field, pretty_print_hex_fields, pretty_print_name_hex_fields,
    validators::rdh::Rdh0Validator, GbtWord,
};
use binrw::{binrw, BinRead};
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::fmt::{self, Debug};
// Newtype pattern used to enforce type safety on fields that are not byte-aligned
#[derive(Debug, PartialEq, Clone, Copy)]
#[binrw]
#[brw(little)]
struct CruidDw(u16); // 12 bit cru_id, 4 bit dw
#[derive(Debug, PartialEq, Clone, Copy)]
#[binrw]
#[brw(little)]
struct BcReserved(u32); // 12 bit bc, 20 bit reserved
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Clone, Copy)]
struct DataformatReserved(u64); // 8 bit data_format, 56 bit reserved0
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FeeId(pub(crate) u16); // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Clone)]
pub struct RdhCRUtest {
    pub rdh0: Rdh0Test,
    pub offset_new_packet: u16,
    pub memory_size: u16,
    pub link_id: u8,
    pub packet_counter: u8,
    cruid_dw: u16, // 12 bit cru_id, 4 bit dw
}
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Rdh0Test {
    // Represents 64 bit
    pub header_id: u8,
    pub header_size: u8,
    pub fee_id: u16, // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number
    pub priority_bit: u8,
    pub system_id: u8,
    pub reserved0: u16,
}

#[binrw]
#[brw(little)]
#[derive(PartialEq, Clone, Copy)]
pub struct RdhCRUv7 {
    pub rdh0: Rdh0,
    pub offset_new_packet: u16,
    pub memory_size: u16,
    pub link_id: u8,
    pub packet_counter: u8,
    cruid_dw: CruidDw, // 12 bit cru_id, 4 bit dw
    pub rdh1: Rdh1,
    dataformat_reserved0: DataformatReserved, // 8 bit data_format, 56 bit reserved0
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
        ((self.cruid_dw.0 & 0xF000) >> 12).try_into().unwrap()
    }
    pub fn data_format(&self) -> u8 {
        // Get the data_format present in the 8 LSB
        (self.dataformat_reserved0.0 & 0x00000000000000FF)
            .try_into()
            .unwrap()
    }
    pub fn reserved0(&self) -> u64 {
        // Get the reserved0 present in the 56 MSB
        (self.dataformat_reserved0.0 & 0xFFFFFFFFFFFFFF00) >> 8
    }
}

impl ByteSlice for RdhCRUv7 {
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { crate::any_as_u8_slice(self) }
    }
}

impl GbtWord for RdhCRUv7 {
    fn print(&self) {
        println!("===================\nRdhCRU:");
        self.rdh0.print();
        let tmp_offset_new_packet = self.offset_new_packet;
        pretty_print_hex_field!("offset_new_packet", tmp_offset_new_packet);
        let tmp_memory_size = self.memory_size;
        pretty_print_hex_field!("memory_size", tmp_memory_size);
        pretty_print_hex_field!("link_id", self.link_id);
        pretty_print_hex_field!("packet_counter", self.packet_counter);
        pretty_print_hex_field!("cru_id", self.cru_id());
        pretty_print_hex_field!("dw", self.dw());
        self.rdh1.print();
        pretty_print_hex_field!("data_format", self.data_format());
        pretty_print_hex_field!("reserved0", self.reserved0());
        self.rdh2.print();
        pretty_print_hex_fields!(self, reserved1);
        self.rdh3.print();
        pretty_print_hex_fields!(self, reserved2);
    }
    fn load<T: std::io::Read>(reader: &mut T) -> Result<RdhCRUv7, std::io::Error> {
        let rdh0 = match Rdh0::load(reader) {
            Ok(rdh0) => rdh0,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Err(std::io::ErrorKind::UnexpectedEof.into())
            }
            Err(e) => return Err(e),
        };
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

        write!(f, "RdhCRUv7: rdh0: {:?}, offset_new_packet: {:x?}, memory_size: {:x?}, link_id: {:x?}, packet_counter: {:x?}, cruid: {:x?}, dw: {:x?}, rdh1: {:?}, data_format: {:x?}, reserved0: {:x?}, rdh2: {:?}, reserved1: {:x?}, rdh3: {:?}, reserved2: {:x?}",
               tmp_rdh0, tmp_offset_new_packet, tmp_memory_size, tmp_link_id, tmp_packet_counter, tmp_cruid, tmp_dw, tmp_rdh1, tmp_data_format, tmp_reserved0, tmp_rdh2, tmp_reserved1, tmp_rdh3, tmp_reserved2)
    }
}

#[binrw]
#[brw(little)]
#[derive(PartialEq, Clone, Copy)]
pub struct RdhCRUv6 {
    pub rdh0: Rdh0,
    pub offset_new_packet: u16,
    pub memory_size: u16,
    pub link_id: u8,
    pub packet_counter: u8,
    cruid_dw: CruidDw, // 12 bit cru_id, 4 bit dw
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
        ((self.cruid_dw.0 & 0xF000) >> 12).try_into().unwrap()
    }
}

impl ByteSlice for RdhCRUv6 {
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { crate::any_as_u8_slice(self) }
    }
}

impl GbtWord for RdhCRUv6 {
    fn print(&self) {
        println!("===================\nRdhCRU:");
        self.rdh0.print();
        pretty_print_hex_fields!(
            self,
            offset_new_packet,
            memory_size,
            link_id,
            packet_counter
        );
        pretty_print_hex_field!("cru_id", self.cru_id());
        pretty_print_hex_field!("dw", self.dw());
        self.rdh1.print();
        pretty_print_hex_fields!(self, reserved0);
        self.rdh2.print();
        pretty_print_hex_fields!(self, reserved1);
        self.rdh3.print();
        pretty_print_hex_fields!(self, reserved2);
    }
    fn load<T: std::io::Read>(reader: &mut T) -> Result<RdhCRUv6, std::io::Error> {
        let rdh0 = match Rdh0::load(reader) {
            Ok(rdh0) => rdh0,
            Err(e) => return Err(e),
        };
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

        write!(f, "RdhCRUv7: rdh0: {:?}, offset_new_packet: {:x?}, memory_size: {:x?}, link_id: {:x?}, packet_counter: {:x?}, cruid: {:x?}, dw: {:x?}, rdh1: {:?}, reserved0: {:x?}, rdh2: {:?}, reserved1: {:x?}, rdh3: {:?}, reserved2: {:x?}",
               tmp_rdh0, tmp_offset_new_packet, tmp_memory_size, tmp_link_id, tmp_packet_counter, tmp_cruid, tmp_dw, tmp_rdh1, tmp_reserved0, tmp_rdh2, tmp_reserved1, tmp_rdh3, tmp_reserved2)
    }
}
#[binrw]
#[brw(little)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Rdh0 {
    // Represents 64 bit
    pub header_id: u8,
    pub header_size: u8,
    pub fee_id: FeeId, // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number
    pub priority_bit: u8,
    pub system_id: u8,
    pub reserved0: u16,
}

impl GbtWord for Rdh0 {
    fn print(&self) {
        println!("Rdh0:");
        pretty_print_hex_field!("header_id", self.header_id);
        pretty_print_hex_field!("header_size", self.header_size);
        let tmp_fee_id = self.fee_id.0;
        pretty_print_hex_field!("fee_id", tmp_fee_id);
        pretty_print_hex_field!("priority_bit", self.priority_bit);
        pretty_print_hex_field!("system_id", self.system_id);
        let tmp_reserved0 = self.reserved0;
        pretty_print_hex_field!("reserved0", tmp_reserved0);
    }
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Rdh0, std::io::Error> {
        // Create a helper macro for loading an array of the given size from
        // the reader.
        macro_rules! load_part {
            // Take an option `size` literal as a parameter
            ($size:literal) => {
                // The whole body goes into a scope so that it is a valid
                // expression when the macro gets expanded.
                {
                    // Create a buffer array of the given size
                    let mut buf = [0u8; $size];
                    // Read into the buffer
                    reader.read_exact(&mut buf)?;
                    // The buffer
                    buf
                }
            };
        }
        // Now we construct return it
        Ok(Rdh0 {
            header_id: load_part!(1)[0],
            header_size: load_part!(1)[0],
            fee_id: FeeId(LittleEndian::read_u16(&load_part!(2))),
            priority_bit: load_part!(1)[0],
            system_id: load_part!(1)[0],
            reserved0: LittleEndian::read_u16(&load_part!(2)),
        })
    }
}
impl Rdh0 {
    pub fn sanity_check(&self, validator: &Rdh0Validator) -> Result<(), GbtError> {
        match validator.sanity_check(self) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
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

        write!(f, "Rdh0: header_id: {:x?}, header_size: {:x?}, fee_id: {:x?}, priority_bit: {:x?}, system_id: {:x?}, reserved0: {:x?}",
               tmp_header_id, tmp_header_size, tmp_fee_id, tmp_priority_bit, tmp_system_id, tmp_reserved0)
    }
}

#[binrw]
#[brw(little)]
#[derive(PartialEq, Clone, Copy)]
pub struct Rdh1 {
    // Rdh1 is 64 bit total
    bc_reserved0: BcReserved, //bunch counter 12 bit + reserved 20 bit
    pub orbit: u32,           // 32 bit
}

impl Rdh1 {
    pub fn bc(&self) -> u16 {
        (self.bc_reserved0.0 & 0x0FFF).try_into().unwrap()
    }
    pub fn reserved0(&self) -> u32 {
        self.bc_reserved0.0 >> 12
    }
}

impl GbtWord for Rdh1 {
    fn print(&self) {
        println!("Rdh1:");
        pretty_print_hex_field!("bc", self.bc());
        pretty_print_hex_field!("reserved0", self.reserved0());
        let tmp_orbit = self.orbit;
        pretty_print_hex_field!("orbit", tmp_orbit);
    }
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Rdh1, std::io::Error> {
        // Create a helper macro for loading an array of the given size from
        // the reader.
        macro_rules! load_bytes {
            // Take an option `size` which is a literal as a parameter
            ($size:literal) => {{
                // Create a buffer array of the given size
                let mut buf = [0u8; $size];
                // Read into the buffer
                reader.read_exact(&mut buf).unwrap();
                // The buffer
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
            "Rdh1: bc: {:x?}, reserved0: {:x?}, orbit: {:x?}",
            tmp_bc, tmp_reserved0, tmp_orbit
        )
    }
}

#[binrw]
#[brw(little)]
#[derive(PartialEq, Clone, Copy)]
pub struct Rdh2 {
    pub trigger_type: u32, // 32 bit
    pub pages_counter: u16,
    pub stop_bit: u8,
    pub reserved0: u8,
}
impl GbtWord for Rdh2 {
    fn print(&self) {
        pretty_print_name_hex_fields!(Rdh2, self, trigger_type, pages_counter, stop_bit, reserved0);
    }
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Rdh2, std::io::Error> {
        // Create a helper macro for loading an array of the given size from
        // the reader.
        macro_rules! load_bytes {
            // Take an option `size`
            ($size:literal) => {{
                // Create a buffer array of the given size
                let mut buf = [0u8; $size];
                // Read into the buffer
                reader.read_exact(&mut buf).unwrap();
                // The buffer
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
            "Rdh2: trigger_type: {:x?}, pages_counter: {:x?}, stop_bit: {:x?}, reserved0: {:x?}",
            tmp_trigger_type, tmp_pages_counter, tmp_stop_bit, tmp_reserved0
        )
    }
}

#[binrw]
#[brw(little)]
#[derive(PartialEq, Clone, Copy)]
pub struct Rdh3 {
    pub detector_field: u32,
    pub par_bit: u16,
    pub reserved0: u16,
}
impl GbtWord for Rdh3 {
    fn print(&self) {
        pretty_print_name_hex_fields!(Rdh3, self, detector_field, par_bit, reserved0);
    }
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Rdh3, std::io::Error> {
        // Create a helper macro for loading an array of the given size from
        // the reader.
        macro_rules! load_bytes {
            // Take an option `size`
            ($size:literal) => {{
                // Create a buffer array of the given size
                let mut buf = [0u8; $size];
                // Read into the buffer
                reader.read_exact(&mut buf).unwrap();
                // The buffer
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
impl Debug for Rdh3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // To align the output, when printing a packed struct, temporary variables are needed
        let tmp_df = self.detector_field;
        let tmp_par = self.par_bit;
        let tmp_res = self.reserved0;
        write!(
            f,
            "Rdh3: detector_field: {:x?}, par_bit: {:x?}, reserved0: {:x?}",
            tmp_df, tmp_par, tmp_res
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::file_open_read_only;
    use crate::validators::rdh;
    use binrw::{BinRead, BinWrite};

    use super::*;
    use std::fs::{self, File, OpenOptions};
    use std::{io::BufReader, path::PathBuf};

    // Verifies that the RdhCruv7 struct is serialized and deserialized correctly
    #[test]
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
        correct_rdh_cru.write(&mut file).unwrap();
        // Open and read with the manual serialization method
        let file = file_open_read_only(&filepath).unwrap();
        let mut buf_reader = BufReader::new(file);
        let rdh_cru = RdhCRUv7::load(&mut buf_reader).expect("Failed to load RdhCRUv7");
        // Open and read with the binrw serialization implementation
        let file = file_open_read_only(&filepath).unwrap();
        let mut buf_reader = BufReader::new(file);
        let rdh_cru_binrw = RdhCRUv7::read(&mut buf_reader).expect("Failed to load RdhCRUv7");
        // Assert that the two methods are equal
        assert_eq!(rdh_cru, correct_rdh_cru);
        assert_eq!(rdh_cru_binrw, correct_rdh_cru);
        // This function currently corresponds to the unlink function on Unix and the DeleteFile function on Windows. Note that, this may change in the future.
        // More info: https://doc.rust-lang.org/std/fs/fn.remove_file.html
        fs::remove_file(&filepath).unwrap();
    }

    // Verifies that the RdhCruv6 struct is serialized and deserialized correctly
    #[test]
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
        correct_rdhcruv6.write(&mut file).unwrap();

        // Open, and read the file manual way
        let file = OpenOptions::new().read(true).open(&filepath).unwrap();
        let mut buf_reader = BufReader::new(file);
        let rdh_cru = RdhCRUv6::load(&mut buf_reader).expect("Failed to load RdhCRUv6");
        // Open and read the file using binrw
        let file_binrw = File::options().read(true).open(&filepath).unwrap();
        let mut buf_reader_binrw = BufReader::new(file_binrw);
        let rdh_cru_binrw = RdhCRUv6::read(&mut buf_reader_binrw).expect("Failed to load RdhCRUv6");

        // Check that both the manual and binrw way of reading the file are correct
        assert_eq!(rdh_cru, correct_rdhcruv6);
        assert_eq!(rdh_cru_binrw, correct_rdhcruv6);

        // This function currently corresponds to the unlink function on Unix and the DeleteFile function on Windows. Note that, this may change in the future.
        // More info: https://doc.rust-lang.org/std/fs/fn.remove_file.html
        fs::remove_file(&filepath).unwrap();
    }

    #[test]
    fn test_binrw_write() {
        let rdh_test = RdhCRUtest {
            rdh0: Rdh0Test {
                header_id: 0x7,
                header_size: 0x40,
                fee_id: 0x502A,
                priority_bit: 0x0,
                system_id: 0x20,
                reserved0: 0,
            },
            offset_new_packet: 0x13E0,
            memory_size: 0x13E0,
            link_id: 0x0,
            packet_counter: 0x0,
            cruid_dw: 0x0018,
        };

        let filename = "test_files/test1.raw";
        let filepath = PathBuf::from(filename);
        match filepath.exists() {
            true => {
                fs::File::options().read(true).open(&filepath).unwrap();
                ()
            }
            false => {
                let mut file = fs::File::create(&filepath).unwrap();
                rdh_test.write(&mut file).expect("Failed to write")
            }
        }
    }

    #[test]
    fn test_binrw_read() {
        let rdh_test = RdhCRUtest {
            rdh0: Rdh0Test {
                header_id: 0x7,
                header_size: 0x40,
                fee_id: 0x502A,
                priority_bit: 0x0,
                system_id: 0x20,
                reserved0: 0,
            },
            offset_new_packet: 0x13E0,
            memory_size: 0x13E0,
            link_id: 0x0,
            packet_counter: 0x0,
            cruid_dw: 0x0018,
        };

        let filename = "test_files/test1.raw";
        let filepath = PathBuf::from(filename);

        let file = match filepath.exists() {
            true => file_open_read_only(&filepath).unwrap(),
            false => {
                while filepath.exists() == false {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
                file_open_read_only(&filepath).unwrap()
            }
        };

        let mut buf_reader = std::io::BufReader::new(file);

        let t = RdhCRUtest::read(&mut buf_reader).unwrap();

        assert_eq!(t, rdh_test);
    }

    #[test]
    fn test_binrw_rdh0() {
        let rdh0 = Rdh0 {
            header_id: 0x7,
            header_size: 0x40,
            fee_id: FeeId(0x502A),
            priority_bit: 0x0,
            system_id: 0x20,
            reserved0: 0,
        };

        let filename = "test_files/rdh0.raw";
        let filepath = PathBuf::from(filename);
        let mut file = match filepath.exists() {
            true => fs::File::options()
                .read(true)
                .write(true)
                .open(&filepath)
                .unwrap(),
            false => {
                let mut f_write = fs::File::create(&filepath).unwrap();
                rdh0.write(&mut f_write).expect("Failed to write");
                std::io::Write::flush(&mut f_write).unwrap();
                let file = fs::File::options().read(true).open(&filepath).unwrap();
                file
            }
        };

        let rdh0_read = Rdh0::read(&mut file).unwrap();
        assert_eq!(rdh0, rdh0_read);
    }
}
