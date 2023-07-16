//! Contains the definition of the [RDH CRU][RdhCru].
use super::rdh0::Rdh0;
use super::rdh1::Rdh1;
use super::rdh2::Rdh2;
use super::rdh3::Rdh3;
use super::{ByteSlice, RdhSubword, SerdeRdh, RDH, RDH_CRU};
use byteorder::{ByteOrder, LittleEndian};
use std::fmt::{self, Display};
use std::{fmt::Debug, marker::PhantomData};

/// Represents the `Data format` and `reserved` fields. Using a newtype because the fields are packed in 64 bits, and extracting the values requires some work.
#[repr(packed)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct DataformatReserved(pub u64); // 8 bit data_format, 56 bit reserved0

/// Represents the `CRU ID` and `DW` fields. Using a newtype because the fields are packed in 16 bits, and extracting the values requires some work.
#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(packed)]
pub struct CruidDw(pub u16); // 12 bit cru_id, 4 bit dw
/// Unit struct to mark a [RdhCru] as version 6.
pub struct V6;
/// Unit struct to mark a [RdhCru] as version 7.
pub struct V7;

/// The struct definition of the [RDH CRU][RdhCru].
///
/// [PhantomData] is used to mark the version of the [RDH CRU][RdhCru]. It's a zero cost abstraction.
/// Among other things, it allows to have different implementations of the [RdhCru] for different versions, but prevents the user from mixing them up.
#[repr(packed)]
pub struct RdhCru<Version> {
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

impl<Version> Display for RdhCru<Version> {
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

impl<Version> RdhCru<Version> {
    /// Creates a new [RDH](RdhCru).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rdh0: Rdh0,
        offset_new_packet: u16,
        memory_size: u16,
        link_id: u8,
        packet_counter: u8,
        cruid_dw: CruidDw,
        rdh1: Rdh1,
        dataformat_reserved0: DataformatReserved,
        rdh2: Rdh2,
        reserved1: u64,
        rdh3: Rdh3,
        reserved2: u64,
    ) -> Self {
        RdhCru {
            rdh0,
            offset_new_packet,
            memory_size,
            link_id,
            packet_counter,
            cruid_dw,
            rdh1,
            dataformat_reserved0,
            rdh2,
            reserved1,
            rdh3,
            reserved2,
            version: PhantomData,
        }
    }

    /// Formats a [String] containing 2 lines that serve as a header, describing columns of key values for an [RDH CRU][RdhCru].
    ///
    /// Can be used to print a header for a table of [RDH CRU][RdhCru]s.
    /// Takes an [usize] as an argument, which is the number of spaces to indent the 2 lines by.
    #[inline(always)]
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
    #[inline(always)]
    pub fn cru_id(&self) -> u16 {
        // Get the cru_id present in the 12 LSB
        self.cruid_dw.0 & 0x0FFF
    }
    /// Returns the value of the DW field.
    #[inline(always)]
    pub fn dw(&self) -> u8 {
        // Get the dw present in the 4 MSB
        ((self.cruid_dw.0 & 0xF000) >> 12) as u8
    }
    /// Returns the value of the data format field.
    #[inline(always)]
    pub fn data_format(&self) -> u8 {
        // Get the data_format present in the 8 LSB
        (self.dataformat_reserved0.0 & 0x00000000000000FF) as u8
    }
    /// Returns the value of the reserved0 field.
    #[inline(always)]
    pub fn reserved0(&self) -> u64 {
        // Get the reserved0 present in the 56 MSB
        (self.dataformat_reserved0.0 & 0xFFFFFFFFFFFFFF00) >> 8
    }

    /// Returns the value of the reserved1 field.
    #[inline(always)]
    pub fn reserved1(&self) -> u64 {
        self.reserved1
    }

    /// Returns the value of the reserved2 field.
    #[inline(always)]
    pub fn reserved2(&self) -> u64 {
        // Get the reserved0 present in the 56 MSB
        self.reserved2
    }
}

impl<Version> PartialEq for RdhCru<Version> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.to_byte_slice() == other.to_byte_slice()
    }
}

impl<Version> Debug for RdhCru<Version> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_offset = self.offset_new_packet;
        let tmp_memory = self.memory_size;
        let tmp_res1 = self.reserved1;
        let tmp_res2 = self.reserved2;

        write!(
            f,
            "RdhCru\n\t{:?}\n\toffset_new_packet: {tmp_offset:?}\n\tmemory_size: {tmp_memory:?}\n\tlink_id: {:?}\n\tpacket_counter: {:?}\n\tcruid_dw: {:?}\n\t{:?}\n\tdataformat_reserved0: {:?}\n\t{:?}\n\treserved1: {tmp_res1:?}\n\t{:?}\n\treserved2: {tmp_res2:?}\n\tversion: {:?}",
            self.rdh0 ,self.link_id, self.packet_counter, self.cruid_dw, self.rdh1, self.dataformat_reserved0, self.rdh2, self.rdh3, self.version
        )
    }
}

impl<Version: Send + Sync> RDH for RdhCru<Version> {}

impl<Version: Send + Sync> RDH_CRU for RdhCru<Version> {
    #[inline(always)]
    fn link_id(&self) -> u8 {
        self.link_id
    }
    #[inline(always)]
    fn payload_size(&self) -> u16 {
        self.memory_size - 64 // 64 bytes are the RDH size. Payload size is the memory size minus the RDH size.
    }
    #[inline(always)]
    fn offset_to_next(&self) -> u16 {
        self.offset_new_packet
    }
    #[inline(always)]
    fn stop_bit(&self) -> u8 {
        self.rdh2.stop_bit
    }
    #[inline(always)]
    fn pages_counter(&self) -> u16 {
        self.rdh2.pages_counter
    }
    #[inline(always)]
    fn data_format(&self) -> u8 {
        self.data_format()
    }
    #[inline(always)]
    fn trigger_type(&self) -> u32 {
        self.rdh2.trigger_type
    }
    #[inline(always)]
    fn fee_id(&self) -> u16 {
        self.rdh0.fee_id.0
    }
    #[inline(always)]
    fn version(&self) -> u8 {
        self.rdh0.header_id
    }
    #[inline(always)]
    fn rdh0(&self) -> &Rdh0 {
        &self.rdh0
    }
    #[inline(always)]
    fn rdh1(&self) -> &Rdh1 {
        &self.rdh1
    }
    #[inline(always)]
    fn rdh2(&self) -> &Rdh2 {
        &self.rdh2
    }
    #[inline(always)]
    fn rdh3(&self) -> &Rdh3 {
        &self.rdh3
    }
    #[inline(always)]
    fn cru_id(&self) -> u16 {
        self.cru_id()
    }
    #[inline(always)]
    fn dw(&self) -> u8 {
        self.dw()
    }
    #[inline(always)]
    fn packet_counter(&self) -> u8 {
        self.packet_counter
    }
}

impl<Version: Send + Sync> SerdeRdh for RdhCru<Version> {
    #[inline(always)]
    fn from_rdh0_and_buf(rdh0: Rdh0, buf: &[u8]) -> Result<Self, std::io::Error> {
        Ok(RdhCru {
            rdh0,
            offset_new_packet: LittleEndian::read_u16(&buf[0..=1]),
            memory_size: LittleEndian::read_u16(&buf[2..=3]),
            link_id: buf[4],
            packet_counter: buf[5],
            cruid_dw: CruidDw(LittleEndian::read_u16(&buf[6..=7])),
            rdh1: Rdh1::from_buf(&buf[8..=15])?,
            dataformat_reserved0: DataformatReserved(LittleEndian::read_u64(&buf[16..=23])),
            rdh2: Rdh2::from_buf(&buf[24..=31])?,
            reserved1: LittleEndian::read_u64(&buf[32..=39]),
            rdh3: Rdh3::from_buf(&buf[40..=47])?,
            reserved2: LittleEndian::read_u64(&buf[48..=55]),
            version: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::rdh0::*;
    use super::super::rdh1::*;
    use super::super::rdh2::*;
    use super::super::rdh3::*;
    use super::super::test_data::*;
    use super::*;

    #[test]
    fn test_header_text() {
        let header_text = RdhCru::<V7>::rdh_header_text_with_indent_to_string(7);
        println!("{header_text}");
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
        let rdhv6 = RdhCru::<V6> {
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

        let rdh_v7 = RdhCru::<V7> {
            rdh0: rdh_0,
            offset_new_packet: 0,
            memory_size: 0x40,
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
        let rdh_v7: RdhCru<V7> = CORRECT_RDH_CRU_V7;
        let rdh_v6: RdhCru<V6> = CORRECT_RDH_CRU_V6;
        println!("{}", RdhCru::<V7>::rdh_header_text_with_indent_to_string(7));
        println!("{rdh_v7}");
        println!("{rdh_v6}");
        let v = rdh_v7.version;
        println!("{v:?}");
        print_rdh_cru_v6(rdh_v6);
        print_rdh_cru(rdh_v7);
        println!("{}", RdhCru::<V7>::rdh_header_text_with_indent_to_string(7));
        let rdh_v7: RdhCru<V7> = CORRECT_RDH_CRU_V7;
        let rdh_v6: RdhCru<V6> = CORRECT_RDH_CRU_V6;
        print_rdh_cru::<V6>(rdh_v6);
        print_rdh_cru::<V7>(rdh_v7);
    }

    fn print_rdh_cru<V>(rdh: RdhCru<V>) {
        println!("{rdh}");
    }
    fn print_rdh_cru_v6(rdh: RdhCru<V6>) {
        println!("{rdh}");
    }

    // Test from old implementation

    #[test]
    fn test_load_rdhcruv7_from_byte_slice() {
        // Create an instace of an RDH-CRU v7
        // byte slice values taken from a valid rdh from real data
        let rdhcruv7 = RdhCru::<V7>::load(
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

        let rdh_from_old = RdhCru::load(&mut rdhcruv7.to_byte_slice()).unwrap();
        let rdh_inferred_from_old = RdhCru::load(&mut rdhcruv7.to_byte_slice()).unwrap();
        let rdh_v7_from_old = RdhCru::<V7>::load(&mut rdhcruv7.to_byte_slice()).unwrap();
        println!("{rdh_from_old}");
        assert_eq!(rdhcruv7, rdh_from_old);
        assert_eq!(rdhcruv7.rdh0.header_size, 0x40);
        assert_ne!(rdhcruv7, CORRECT_RDH_CRU_V7);
        assert_eq!(rdh_inferred_from_old, rdh_v7_from_old);
        dbg!(rdhcruv7);
    }
}
