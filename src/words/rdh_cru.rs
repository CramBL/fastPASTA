//! Contains the definition of the [RDH CRU][RdhCRU].
use super::lib::{ByteSlice, RdhSubWord};
use crate::words::rdh::{CruidDw, DataformatReserved, Rdh0, Rdh1, Rdh2, Rdh3};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fmt::{self, Display};
use std::{fmt::Debug, marker::PhantomData};
/// Unit struct to mark a [RdhCRU] as version 6.
pub struct V6;
/// Unit struct to mark a [RdhCRU] as version 7.
pub struct V7;

/// The struct definition of the [RDH CRU][RdhCRU].
///
/// [PhantomData] is used to mark the version of the [RDH CRU][RdhCRU]. It's a zero cost abstraction.
/// Among other things, it allows to have different implementations of the [RdhCRU] for different versions, but prevents the user from mixing them up.
#[repr(packed)]
pub struct RdhCRU<Version> {
    pub(crate) rdh0: Rdh0,
    pub(crate) offset_new_packet: u16,
    pub(crate) memory_size: u16,
    pub(crate) link_id: u8,
    pub(crate) packet_counter: u8,
    pub(crate) cruid_dw: CruidDw, // 12 bit cru_id, 4 bit dw
    pub(crate) rdh1: Rdh1,
    pub(crate) dataformat_reserved0: DataformatReserved, // 8 bit data_format, 56 bit reserved0
    pub(crate) rdh2: Rdh2,
    pub(crate) reserved1: u64,
    pub(crate) rdh3: Rdh3,
    pub(crate) reserved2: u64,
    pub(crate) version: PhantomData<Version>,
}

impl<Version> Display for RdhCRU<Version> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_offset = self.offset_new_packet;
        let tmp_link = self.link_id;
        let tmp_packet_cnt = self.packet_counter;
        let rdhcru_fields0 = format!("{tmp_offset:<8}{tmp_link:<6}{tmp_packet_cnt:<10}");
        write!(
            f,
            "{}{rdhcru_fields0}{}{:<11}{}",
            self.rdh0,
            self.rdh1,
            self.data_format(),
            self.rdh2
        )
    }
}

impl<Version> RdhCRU<Version> {
    /// Formats a [String] containing 2 lines that serve as a header, describing columns of key values for an [RDH CRU][RdhCRU].
    ///
    /// Can be used to print a header for a table of [RDH CRU][RdhCRU]s.
    /// Takes an [usize] as an argument, which is the number of spaces to indent the 2 lines by.
    #[inline]
    pub fn rdh_header_text_with_indent_to_string(indent: usize) -> String {
        let header_text_top = "RDH   Header  FEE   Sys   Offset  Link  Packet    BC   Orbit       Data       Trigger   Pages    Stop";
        let header_text_bottom = "ver   size    ID    ID    next    ID    counter        counter     format     type      counter  bit";
        format!(
            "{:indent$}{header_text_top}\n{:indent2$}{header_text_bottom}\n",
            "",
            "",
            indent = indent,
            indent2 = indent
        )
    }
    /// Returns the value of the CRU ID field.
    #[inline]
    pub fn cru_id(&self) -> u16 {
        // Get the cru_id present in the 12 LSB
        self.cruid_dw.0 & 0x0FFF
    }
    /// Returns the value of the DW field.
    #[inline]
    pub fn dw(&self) -> u8 {
        // Get the dw present in the 4 MSB
        ((self.cruid_dw.0 & 0xF000) >> 12) as u8
    }
    /// Returns the value of the data format field.
    #[inline]
    pub fn data_format(&self) -> u8 {
        // Get the data_format present in the 8 LSB
        (self.dataformat_reserved0.0 & 0x00000000000000FF) as u8
    }
    /// Returns the value of the reserved0 field.
    #[inline]
    pub fn reserved0(&self) -> u64 {
        // Get the reserved0 present in the 56 MSB
        (self.dataformat_reserved0.0 & 0xFFFFFFFFFFFFFF00) >> 8
    }
}

impl<Version> PartialEq for RdhCRU<Version> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.to_byte_slice() == other.to_byte_slice()
    }
}

impl<Version> Debug for RdhCRU<Version> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_offset = self.offset_new_packet;
        let tmp_memory = self.memory_size;
        let tmp_res1 = self.reserved1;
        let tmp_res2 = self.reserved2;

        write!(
            f,
            "RdhCRU\n\t{:?}\n\toffset_new_packet: {tmp_offset:?}\n\tmemory_size: {tmp_memory:?}\n\tlink_id: {:?}\n\tpacket_counter: {:?}\n\tcruid_dw: {:?}\n\t{:?}\n\tdataformat_reserved0: {:?}\n\t{:?}\n\treserved1: {tmp_res1:?}\n\t{:?}\n\treserved2: {tmp_res2:?}\n\tversion: {:?}",
            self.rdh0 ,self.link_id, self.packet_counter, self.cruid_dw, self.rdh1, self.dataformat_reserved0, self.rdh2, self.rdh3, self.version
        )
    }
}

impl<Version: std::marker::Send + std::marker::Sync> super::lib::RDH for RdhCRU<Version> {
    #[inline]
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let rdh0 = match Rdh0::load(reader) {
            Ok(rdh0) => rdh0,
            Err(e) => return Err(e),
        };
        Self::load_from_rdh0(reader, rdh0)
    }
    #[inline]
    fn load_from_rdh0<T: std::io::Read>(
        reader: &mut T,
        rdh0: Rdh0,
    ) -> Result<Self, std::io::Error> {
        let offset_new_packet = reader.read_u16::<LittleEndian>()?;
        let memory_size = reader.read_u16::<LittleEndian>()?;
        let link_id = reader.read_u8()?;
        let packet_counter = reader.read_u8()?;
        // cru_id is 12 bit and the following dw is 4 bit
        let tmp_cruid_dw = CruidDw(reader.read_u16::<LittleEndian>()?);
        let rdh1 = Rdh1::load(reader)?;
        // Now the next 64 bits contain the reserved0 and data_format
        // [7:0]data_format, [63:8]reserved0
        let tmp_dataformat_reserverd0 = DataformatReserved(reader.read_u64::<LittleEndian>()?);
        let rdh2 = Rdh2::load(reader)?;
        let reserved1 = reader.read_u64::<LittleEndian>()?;
        let rdh3 = Rdh3::load(reader)?;
        let reserved2 = reader.read_u64::<LittleEndian>()?;
        // Finally return the RdhCRU
        Ok(RdhCRU {
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
            version: PhantomData,
        })
    }

    #[inline]
    fn link_id(&self) -> u8 {
        self.link_id
    }
    #[inline]
    fn payload_size(&self) -> u16 {
        self.memory_size - 64 // 64 bytes are the RDH size. Payload size is the memory size minus the RDH size.
    }
    #[inline]
    fn offset_to_next(&self) -> u16 {
        self.offset_new_packet
    }
    #[inline]
    fn stop_bit(&self) -> u8 {
        self.rdh2.stop_bit
    }
    #[inline]
    fn pages_counter(&self) -> u16 {
        self.rdh2.pages_counter
    }
    #[inline]
    fn data_format(&self) -> u8 {
        self.data_format()
    }
    #[inline]
    fn trigger_type(&self) -> u32 {
        self.rdh2.trigger_type
    }
    #[inline]
    fn fee_id(&self) -> u16 {
        self.rdh0.fee_id.0
    }
    #[inline]
    fn version(&self) -> u8 {
        self.rdh0.header_id
    }
    #[inline]
    fn rdh0(&self) -> &Rdh0 {
        &self.rdh0
    }
    #[inline]
    fn rdh1(&self) -> &Rdh1 {
        &self.rdh1
    }
    #[inline]
    fn rdh2(&self) -> &Rdh2 {
        &self.rdh2
    }
    #[inline]
    fn rdh3(&self) -> &Rdh3 {
        &self.rdh3
    }
    #[inline]
    fn cru_id(&self) -> u16 {
        self.cru_id()
    }
    #[inline]
    fn dw(&self) -> u8 {
        self.dw()
    }
    #[inline]
    fn packet_counter(&self) -> u8 {
        self.packet_counter
    }
}

pub mod test_data {
    //! Contains test data for testing functionality on a [RDH CRU][RdhCRU].
    use crate::words::rdh::{BcReserved, FeeId};

    // For testing
    use super::*;
    /// Convenience struct of a [RDH CRU][RdhCRU] with the version [V7] used in tests.
    pub const CORRECT_RDH_CRU_V7: RdhCRU<V7> = RdhCRU::<V7> {
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
        version: PhantomData,
    };

    /// Convenience struct of a [RDH CRU][RdhCRU] with the version [V6] used in tests.
    pub const CORRECT_RDH_CRU_V6: RdhCRU<V6> = RdhCRU::<V6> {
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
        dataformat_reserved0: DataformatReserved(0),
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
        version: PhantomData,
    };

    /// Convenience struct of an [RDH CRU][RdhCRU] coming after an initial [RDH CRU][RdhCRU] with the version [V7] used in tests.
    pub const CORRECT_RDH_CRU_V7_NEXT: RdhCRU<V7> = RdhCRU::<V7> {
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
        packet_counter: 0x2,
        cruid_dw: CruidDw(0x0018),
        rdh1: Rdh1 {
            bc_reserved0: BcReserved(0x0),
            orbit: 0x0b7dd575,
        },
        dataformat_reserved0: DataformatReserved(0x2),
        rdh2: Rdh2 {
            trigger_type: 0x00006a03,
            pages_counter: 0x1,
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
        version: PhantomData,
    };

    /// Convenience struct of an [RDH CRU][RdhCRU] closing an HBF, used in tests.
    pub const CORRECT_RDH_CRU_V7_NEXT_NEXT_STOP: RdhCRU<V7> = RdhCRU::<V7> {
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
        packet_counter: 0x3,
        cruid_dw: CruidDw(0x0018),
        rdh1: Rdh1 {
            bc_reserved0: BcReserved(0x0),
            orbit: 0x0b7dd575,
        },
        dataformat_reserved0: DataformatReserved(0x2),
        rdh2: Rdh2 {
            trigger_type: 0x00006a03,
            pages_counter: 0x2,
            stop_bit: 0x1,
            reserved0: 0x0,
        },
        reserved1: 0x0,
        rdh3: Rdh3 {
            detector_field: 0x0,
            par_bit: 0x0,
            reserved0: 0x0,
        },
        reserved2: 0x0,
        version: PhantomData,
    };
}

#[cfg(test)]
mod tests {
    use super::test_data::*;
    use super::*;
    use crate::words::{
        lib::RDH,
        rdh::{BcReserved, FeeId},
        rdh_cru,
    };

    #[test]
    fn test_header_text() {
        let header_text = RdhCRU::<V7>::rdh_header_text_with_indent_to_string(7);
        println!("{}", header_text);
    }

    #[test]
    fn test_correct_rdh_fields() {
        let rdh = CORRECT_RDH_CRU_V7;

        assert_eq!(rdh.rdh0.header_id, 0x7);
        assert_eq!(rdh.rdh0.header_size, 0x40);
        assert_eq!(rdh.rdh0.fee_id, FeeId(0x502A));
        assert_eq!(rdh.rdh0.priority_bit, 0x0);
        assert_eq!(rdh.rdh0.system_id, 0x20);
        let pages_counter = rdh.rdh2.pages_counter;
        assert_eq!(pages_counter, 0x0);
        assert_eq!(rdh.rdh2().stop_bit, 0x0);
        let trigger_type = rdh.rdh2.trigger_type;
        assert_eq!(trigger_type, 0x00006a03);
        assert_eq!(rdh.reserved0(), 0);
        assert_eq!(rdh.payload_size(), 0x13E0 - 0x40); // 0x40 is the header size
        assert_eq!(rdh.trigger_type(), 0x00006a03);
        assert_eq!(rdh.pages_counter(), 0);
        assert_eq!(rdh.fee_id(), 0x502A);
        assert_eq!(rdh.version(), 7);
        assert_eq!(rdh.cru_id(), 0x0018);
        assert_eq!(rdh.packet_counter(), 0);
    }

    #[test]
    fn test_rdh_v6() {
        let rdhv6 = RdhCRU::<V6> {
            rdh0: Rdh0 {
                header_size: 0,
                header_id: 0,
                fee_id: FeeId(0),
                priority_bit: 0,
                system_id: 0,
                reserved0: 0,
            },
            offset_new_packet: 0,
            memory_size: 0,
            link_id: 0,
            packet_counter: 0,
            cruid_dw: CruidDw(0),
            rdh1: Rdh1 {
                bc_reserved0: BcReserved(0),
                orbit: 0,
            },
            dataformat_reserved0: DataformatReserved(0),
            rdh2: Rdh2 {
                trigger_type: 0,
                pages_counter: 0,
                stop_bit: 0,
                reserved0: 0,
            },
            reserved1: 0,
            rdh3: Rdh3 {
                detector_field: 0,
                par_bit: 0,
                reserved0: 0,
            },
            reserved2: 0,
            version: PhantomData,
        };
        assert_eq!(rdhv6.data_format(), 0);
    }

    #[test]
    fn test_rdh_v7() {
        let rdh_0 = CORRECT_RDH_CRU_V7.rdh0;

        let rdh_v7 = RdhCRU::<V7> {
            rdh0: rdh_0,
            offset_new_packet: 0,
            memory_size: 0,
            link_id: 0,
            packet_counter: 0,
            cruid_dw: CruidDw(0),
            rdh1: Rdh1 {
                bc_reserved0: BcReserved(0),
                orbit: 0,
            },
            dataformat_reserved0: DataformatReserved(2),
            rdh2: Rdh2 {
                trigger_type: 0,
                pages_counter: 0,
                stop_bit: 0,
                reserved0: 0,
            },
            reserved1: 0,
            rdh3: Rdh3 {
                detector_field: 0,
                par_bit: 0,
                reserved0: 0,
            },
            reserved2: 0,
            version: PhantomData,
        };
        assert_eq!(rdh_v7.data_format(), 2);
        assert_eq!(rdh_v7.cru_id(), 0);
        assert_eq!(rdh_v7.reserved0(), 0);
        assert_eq!(rdh_v7.payload_size(), 0);
    }

    #[test]
    fn test_print_generic() {
        let rdh_v7: RdhCRU<V7> = CORRECT_RDH_CRU_V7;
        let rdh_v6: RdhCRU<V6> = CORRECT_RDH_CRU_V6;
        println!(
            "{}",
            RdhCRU::<rdh_cru::V7>::rdh_header_text_with_indent_to_string(7)
        );
        println!("{rdh_v7}");
        println!("{rdh_v6}");
        let v = rdh_v7.version;
        println!("{:?}", v);
        print_rdh_cru_v6(rdh_v6);
        print_rdh_cru(rdh_v7);
        println!("{}", RdhCRU::<V7>::rdh_header_text_with_indent_to_string(7));
        let rdh_v7: RdhCRU<V7> = CORRECT_RDH_CRU_V7;
        let rdh_v6: RdhCRU<V6> = CORRECT_RDH_CRU_V6;
        print_rdh_cru::<V6>(rdh_v6);
        print_rdh_cru::<V7>(rdh_v7);
    }

    fn print_rdh_cru<V>(rdh: RdhCRU<V>) {
        println!("{rdh}");
    }
    fn print_rdh_cru_v6(rdh: RdhCRU<V6>) {
        println!("{rdh}");
    }

    // Test from old implementation

    #[test]
    fn test_load_rdhcruv7_from_byte_slice() {
        // Create an instace of an RDH-CRU v7
        // byte slice values taken from a valid rdh from real data
        let rdhcruv7 = RdhCRU::<V7>::load(
            &mut &[
                0x07, 0x40, 0x2a, 0x50, 0x00, 0x20, 0x00, 0x00, 0xe0, 0x13, 0xe0, 0x13, 0x00, 0x00,
                0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x75, 0xd5, 0x7d, 0x0b, 0x02, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x6a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ][..],
        )
        .unwrap();
        // Check that the fields are correct
        println!("{rdhcruv7}");

        let rdh_from_old = RdhCRU::load(&mut &rdhcruv7.to_byte_slice()[..]).unwrap();
        let rdh_inferred_from_old = RdhCRU::load(&mut &rdhcruv7.to_byte_slice()[..]).unwrap();
        let rdh_v7_from_old = RdhCRU::<V7>::load(&mut &rdhcruv7.to_byte_slice()[..]).unwrap();
        println!("{rdh_from_old}");
        assert_eq!(rdhcruv7, rdh_from_old);
        assert_eq!(rdhcruv7.rdh0.header_size, 0x40);
        assert_ne!(rdhcruv7, CORRECT_RDH_CRU_V7);
        assert_eq!(rdh_inferred_from_old, rdh_v7_from_old);
        dbg!(rdhcruv7);
    }
}
