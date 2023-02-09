use macros::print::pretty_print_name_hex_fields;
use std::fmt::{self, Debug};

pub mod macros;

pub trait GbtWord: fmt::Debug + PartialEq {
    fn print(&self);
    fn load<T: std::io::Read>(reader: &mut T) -> Self;
}

// ITS data format: https://gitlab.cern.ch/alice-its-wp10-firmware/RU_mainFPGA/-/wikis/ITS%20Data%20Format#Introduction

#[repr(packed)]
#[derive(PartialEq)]
pub struct RdhCRUv7 {
    pub rdh0: Rdh0,
    pub offset_new_packet: u16,
    pub memory_size: u16,
    pub link_id: u8,
    pub packet_counter: u8,
    pub cru_id: u16, // 12 bit
    pub dw: u8,      // data wrapper id: 4 bit
    pub rdh1: Rdh1,
    pub data_format: u8,
    pub reserved0: u64, // 56 bit
    pub rdh2: Rdh2,
    pub reserved1: u64,
    pub rdh3: Rdh3,
    pub reserved2: u64,
}

impl GbtWord for RdhCRUv7 {
    fn load<T: std::io::Read>(reader: &mut T) -> Self {
        use byteorder::{LittleEndian, ReadBytesExt};
        let rdh0 = Rdh0::load(reader);
        let offset_new_packet = reader.read_u16::<LittleEndian>().unwrap();
        let memory_size = reader.read_u16::<LittleEndian>().unwrap();
        let link_id = reader.read_u8().unwrap();
        let packet_counter = reader.read_u8().unwrap();
        // cru_id is 12 bit and the following dw is 4 bit
        let tmp_16_bit = reader.read_u16::<LittleEndian>().unwrap();
        // Take the first 12 bits as cru_id
        let cru_id = tmp_16_bit & 0x0FFF;
        // Take the last 4 bits as dw, convert to u8 and panic! if it fails
        let dw: u8 = ((tmp_16_bit >> 12) & 0x000F).try_into().unwrap();
        let rdh1 = Rdh1::load(reader);
        // Now the next 64 bits contain the reserved0 and data_format
        // [7:0]data_format, [63:8]reserved0
        let tmp_64_bit = reader.read_u64::<LittleEndian>().unwrap();
        // Take the first 8 bits as data_format, convert to u8 and panic! if it doesn't fit
        let data_format = (tmp_64_bit & 0x00000000000000FF).try_into().unwrap();
        // Take the last 56 bits as reserved0
        let reserved0 = tmp_64_bit >> 8;
        let rdh2 = Rdh2::load(reader);
        let reserved1 = reader.read_u64::<LittleEndian>().unwrap();
        let rdh3 = Rdh3::load(reader);
        let reserved2 = reader.read_u64::<LittleEndian>().unwrap();
        // Finally return the RdhCRU
        RdhCRUv7 {
            rdh0,
            offset_new_packet,
            memory_size,
            link_id,
            packet_counter,
            cru_id,
            dw,
            rdh1,
            data_format,
            reserved0,
            rdh2,
            reserved1,
            rdh3,
            reserved2,
        }
    }
    fn print(&self) {
        use macros::print::pretty_print_hex_fields;
        println!("===================\nRdhCRU:");
        self.rdh0.print();
        pretty_print_hex_fields!(
            self,
            offset_new_packet,
            memory_size,
            link_id,
            packet_counter,
            cru_id,
            dw
        );
        self.rdh1.print();
        pretty_print_hex_fields!(self, data_format, reserved0);
        self.rdh2.print();
        pretty_print_hex_fields!(self, reserved1);
        self.rdh3.print();
        pretty_print_hex_fields!(self, reserved2);
    }
}
impl Debug for RdhCRUv7 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_rdh0 = &self.rdh0;
        let tmp_offset_new_packet = self.offset_new_packet;
        let tmp_memory_size = self.memory_size;
        let tmp_link_id = self.link_id;
        let tmp_packet_counter = self.packet_counter;
        let tmp_cru_id = self.cru_id;
        let tmp_dw = self.dw;
        let tmp_rdh1 = &self.rdh1;
        let tmp_data_format = self.data_format;
        let tmp_reserved0 = self.reserved0;
        let tmp_rdh2 = &self.rdh2;
        let tmp_reserved1 = self.reserved1;
        let tmp_rdh3 = &self.rdh3;
        let tmp_reserved2 = self.reserved2;

        write!(f, "RdhCRUv7: rdh0: {:?}, offset_new_packet: {:x?}, memory_size: {:x?}, link_id: {:x?}, packet_counter: {:x?}, cru_id: {:x?}, dw: {:x?}, rdh1: {:?}, data_format: {:x?}, reserved0: {:x?}, rdh2: {:?}, reserved1: {:x?}, rdh3: {:?}, reserved2: {:x?}",
               tmp_rdh0, tmp_offset_new_packet, tmp_memory_size, tmp_link_id, tmp_packet_counter, tmp_cru_id, tmp_dw, tmp_rdh1, tmp_data_format, tmp_reserved0, tmp_rdh2, tmp_reserved1, tmp_rdh3, tmp_reserved2)
    }
}
#[repr(packed)]
#[derive(PartialEq)]
pub struct Rdh0 {
    // Represents 64 bit
    pub header_id: u8,
    pub header_size: u8,
    pub fee_id: u16, // [0]reserved0, [2:0]layer, [1:0]reserved1, [1:0]fiber_uplink, [1:0]reserved2, [5:0]stave_number
    pub priority_bit: u8,
    pub system_id: u8,
    pub reserved0: u16,
}

impl GbtWord for Rdh0 {
    fn load<T: std::io::Read>(reader: &mut T) -> Rdh0 {
        // Create a helper macro for loading an array of the given size from
        // the reader.
        macro_rules! load_part {
            // Take an option `size` which is a literal as a parameter
            ($size:literal) => {
                // The whole body goes into a scope so that it is a valid
                // expression when the macro gets expanded.
                {
                    // Create a buffer array of the given size
                    let mut buf = [0u8; $size];
                    // Read into the buffer
                    reader.read_exact(&mut buf).unwrap();
                    // The buffer
                    buf
                }
            };
        }

        use byteorder::ByteOrder;
        use byteorder::LittleEndian;
        // Now we construct return it
        Rdh0 {
            header_id: load_part!(1)[0],
            header_size: load_part!(1)[0],
            fee_id: LittleEndian::read_u16(&load_part!(2)),
            priority_bit: load_part!(1)[0],
            system_id: load_part!(1)[0],
            reserved0: LittleEndian::read_u16(&load_part!(2)),
        }
    }
    fn print(&self) {
        pretty_print_name_hex_fields!(
            Rdh0,
            self,
            header_id,
            header_size,
            fee_id,
            priority_bit,
            system_id,
            reserved0
        );
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

#[repr(packed)]
#[derive(PartialEq)]
pub struct Rdh1 {
    // Represents 64 bit
    pub bc: u16,        //bunch counter 12 bit
    pub reserved0: u32, // 20 bit
    pub orbit: u32,     // 32 bit
}

impl GbtWord for Rdh1 {
    fn load<T: std::io::Read>(reader: &mut T) -> Self {
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
        use byteorder::ByteOrder;
        use byteorder::LittleEndian;
        // Load the first 2 bytes that contain the bc[11:0] and reserved0[15:12]
        let _first_2_bytes = LittleEndian::read_u16(&load_bytes!(2));
        // Extract the bc[11:0] by masking the first 12 bits
        let tmp_bc: u16 = _first_2_bytes & 0x0FFF;
        // Extract the reserved0[15:12] by masking the last 4 bits and shifting them to the LSB
        let tmp_reserved0: u32 = ((_first_2_bytes & 0xF000) >> 12) as u32;
        // Load the next 2 bytes that contain the 16 MSB of reserved0
        let _next_2_bytes: u16 = LittleEndian::read_u16(&load_bytes!(2));
        // Combine the 16 MSB of reserved0 with the 4 LSB of reserved0
        let tmp_reserved0 = tmp_reserved0 | ((_next_2_bytes as u32) << 4);

        Rdh1 {
            bc: tmp_bc,
            reserved0: tmp_reserved0,
            orbit: LittleEndian::read_u32(&load_bytes!(4)),
        }
    }
    fn print(&self) {
        pretty_print_name_hex_fields!(Rdh1, self, bc, reserved0, orbit);
    }
}
impl Debug for Rdh1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_bc = self.bc;
        let tmp_reserved0 = self.reserved0;
        let tmp_orbit = self.orbit;
        write!(
            f,
            "Rdh1: bc: {:x?}, reserved0: {:x?}, orbit: {:x?}",
            tmp_bc, tmp_reserved0, tmp_orbit
        )
    }
}

#[repr(packed)]
#[derive(PartialEq)]
pub struct Rdh2 {
    pub trigger_type: u32, // 32 bit
    pub pages_counter: u16,
    pub stop_bit: u8,
    pub reserved0: u8,
}
impl GbtWord for Rdh2 {
    fn load<T: std::io::Read>(reader: &mut T) -> Self {
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
        use byteorder::ByteOrder;
        use byteorder::LittleEndian;

        Rdh2 {
            trigger_type: LittleEndian::read_u32(&load_bytes!(4)),
            pages_counter: LittleEndian::read_u16(&load_bytes!(2)),
            stop_bit: load_bytes!(1)[0],
            reserved0: load_bytes!(1)[0],
        }
    }
    fn print(&self) {
        pretty_print_name_hex_fields!(Rdh2, self, trigger_type, pages_counter, stop_bit, reserved0);
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

#[repr(packed)]
#[derive(PartialEq)]
pub struct Rdh3 {
    pub detector_field: u32,
    pub par_bit: u16,
    pub reserved0: u16,
}
impl GbtWord for Rdh3 {
    fn load<T: std::io::Read>(reader: &mut T) -> Self {
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
        use byteorder::ByteOrder;
        use byteorder::LittleEndian;

        Rdh3 {
            detector_field: LittleEndian::read_u32(&load_bytes!(4)),
            par_bit: LittleEndian::read_u16(&load_bytes!(2)),
            reserved0: LittleEndian::read_u16(&load_bytes!(2)),
        }
    }
    fn print(&self) {
        pretty_print_name_hex_fields!(Rdh3, self, detector_field, par_bit, reserved0);
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
    use super::*;
    use std::fs::{File, OpenOptions};
    use std::{io::BufReader, io::Write, path::PathBuf};

    #[test]
    fn test_load_rdhcruv7() {
        let correct_rdh_cru = RdhCRUv7 {
            rdh0: Rdh0 {
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
            cru_id: 0x18,
            dw: 0x0,
            rdh1: Rdh1 {
                bc: 0x0,
                reserved0: 0x0,
                orbit: 0x0b7dd575,
            },
            data_format: 0x2,
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

        let rdh_cru_as_u8_slice = unsafe {
            ::core::slice::from_raw_parts(
                &correct_rdh_cru as *const RdhCRUv7 as *const u8,
                ::core::mem::size_of::<RdhCRUv7>(),
            )
        };

        let filepath = PathBuf::from("test_rdh_cru");
        let mut file = File::create(&filepath).unwrap();
        file.write_all(&rdh_cru_as_u8_slice).unwrap();

        let file = OpenOptions::new().read(true).open(&filepath).unwrap();
        let mut buf_reader = BufReader::new(file);
        let rdh_cru = RdhCRUv7::load(&mut buf_reader);

        assert_eq!(rdh_cru.rdh0, correct_rdh_cru.rdh0);
        assert_eq!(
            rdh_cru.offset_new_packet.to_le_bytes(),
            correct_rdh_cru.offset_new_packet.to_le_bytes()
        );
        assert_eq!(
            rdh_cru.memory_size.to_le_bytes(),
            correct_rdh_cru.memory_size.to_le_bytes()
        );
        assert_eq!(rdh_cru.link_id, correct_rdh_cru.link_id);
        assert_eq!(rdh_cru.packet_counter, correct_rdh_cru.packet_counter);
        // Breaks after cru_id as it is written to disk as a 16 bit value
        // but actually is only 12 bits, and the other 4 bits should repressent dw
        // Instead they are broken up into a 16 bit value and an 8 bit value, which causes misalignment
        assert_eq!(
            rdh_cru.cru_id.to_le_bytes(),
            correct_rdh_cru.cru_id.to_le_bytes()
        );
        // only works because dw is 0 in this case
        assert_eq!(rdh_cru.dw, correct_rdh_cru.dw);
    }
}
