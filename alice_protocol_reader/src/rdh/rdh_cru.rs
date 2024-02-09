//! Contains the definition of the [RDH CRU][RdhCru].
use super::rdh0::Rdh0;
use super::rdh1::Rdh1;
use super::rdh2::Rdh2;
use super::rdh3::Rdh3;
use super::{ByteSlice, RdhSubword, SerdeRdh, RDH, RDH_CRU};
use byteorder::{ByteOrder, LittleEndian};
use std::fmt::Debug;
use std::fmt::{self, Display};

const HEADER_TEXT_TOP: [&str; 14] = [
    "RDH   ",
    "Header ",
    "FEE    ",
    "Sys   ",
    "Offset  ",
    "Link  ",
    "Packet    ",
    "BC   ",
    "Orbit       ",
    "Data       ",
    "Trigger   ",
    "Pages    ",
    "Stop  ",
    "Detector  ",
];
const HEADER_TEXT_BOT: [&str; 14] = [
    "ver   ",
    "size   ",
    "ID     ",
    "ID    ",
    "next    ",
    "ID    ",
    "counter   ",
    "     ",
    "counter     ",
    "format     ",
    "type      ",
    "counter  ",
    "bit   ",
    "field     ",
];

/// Represents the `Data format` and `reserved` fields. Using a newtype because the fields are packed in 64 bits, and extracting the values requires some work.
#[repr(packed)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct DataformatReserved(pub u64); // 8 bit data_format, 56 bit reserved0

/// Represents the `CRU ID` and `DW` fields. Using a newtype because the fields are packed in 16 bits, and extracting the values requires some work.
#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(packed)]
pub struct CruidDw(pub u16); // 12 bit cru_id, 4 bit dw

/// The struct definition of the [RDH CRU][RdhCru].
///
/// Among other things, it allows to have different implementations of the [RdhCru] for different versions, but prevents the user from mixing them up.
#[allow(missing_copy_implementations)]
#[repr(packed)]
pub struct RdhCru {
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
}

impl Display for RdhCru {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_offset = self.offset_new_packet;
        let tmp_link = self.link_id;
        let tmp_packet_cnt = self.packet_counter;
        let rdhcru_fields0 = format!("{tmp_offset:<8}{tmp_link:<6}{tmp_packet_cnt:<10}");
        let detector_field = self.rdh3.detector_field;
        write!(
            f,
            "{rdh0}{rdhcru_fields0}{rdh1}{data_format:<11}{rdh2} {det_field:#x}",
            rdh0 = self.rdh0,
            rdh1 = self.rdh1,
            data_format = self.data_format(),
            rdh2 = self.rdh2,
            det_field = detector_field
        )
    }
}

impl RdhCru {
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
        }
    }

    /// Formats a [String] containing 2 lines that serve as a header, describing columns of key values for an [RDH CRU][RdhCru].
    ///
    /// Can be used to print a header for a table of [RDH CRU][RdhCru]s.
    /// Takes an [usize] as an argument, which is the number of spaces to indent the 2 lines by.
    /// the columns are styled with alternating background colors.
    #[inline]
    pub fn rdh_header_styled_text_with_indent_to_string(indent: usize) -> String {
        use owo_colors::OwoColorize;

        let (top_text, bot_text) = {
            let mut top_text = String::new();
            let mut bot_text = String::new();
            HEADER_TEXT_BOT
                .iter()
                .zip(HEADER_TEXT_TOP.iter())
                .enumerate()
                .for_each(|(idx, (bot, top))| {
                    // Alternate between on_green and on_blue and append 2 spaces
                    if idx % 2 == 0 {
                        top_text.push_str(&format!("{}", top.white().bold().bg_rgb::<0, 99, 0>()));
                        bot_text.push_str(&format!("{}", bot.white().bold().bg_rgb::<0, 99, 0>()));
                    } else {
                        top_text.push_str(&format!("{}", top.white().bold().bg_rgb::<0, 0, 99>()));
                        bot_text.push_str(&format!("{}", bot.white().bold().bg_rgb::<0, 0, 99>()));
                    }
                });
            (top_text, bot_text)
        };
        format!(
            "{:indent$}{top_text}\n{:indent2$}{bot_text}\n",
            "",
            "",
            indent = indent,
            indent2 = indent,
        )
    }

    /// Formats a [String] containing 2 lines that serve as a header, describing columns of key values for an [RDH CRU][RdhCru].
    ///
    /// Can be used to print a header for a table of [RDH CRU][RdhCru]s.
    /// Takes an [usize] as an argument, which is the number of spaces to indent the 2 lines by.
    #[inline]
    pub fn rdh_header_text_with_indent_to_string(indent: usize) -> String {
        let header_text_top = "RDH   Header  FEE   Sys   Offset  Link  Packet    BC   Orbit       Data       Trigger   Pages    Stop  Detector";
        let header_text_bot = "ver   size    ID    ID    next    ID    counter        counter     format     type      counter  bit   field";
        format!(
            "{:indent$}{header_text_top}\n{:indent2$}{header_text_bot}\n",
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

    /// Returns the value of the reserved1 field.
    #[inline]
    pub fn reserved1(&self) -> u64 {
        self.reserved1
    }

    /// Returns the value of the reserved2 field.
    #[inline]
    pub fn reserved2(&self) -> u64 {
        // Get the reserved0 present in the 56 MSB
        self.reserved2
    }
}

impl PartialEq for RdhCru {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.to_byte_slice() == other.to_byte_slice()
    }
}

impl Debug for RdhCru {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp_offset = self.offset_new_packet;
        let tmp_memory = self.memory_size;
        let tmp_res1 = self.reserved1;
        let tmp_res2 = self.reserved2;
        let tmp_cruid_dw = self.cruid_dw.0;
        let tmp_dataformat_reserved0 = self.dataformat_reserved0.0;
        write!(
            f,
            "RdhCru\n\t{rdh0:?}\n\toffset_new_packet: {tmp_offset:?}\n\tmemory_size: {tmp_memory:?}\n\tlink_id: {link_id:?}\n\tpacket_counter: {packet_counter:?}\n\tcruid_dw: {cruid_dw:?}\n\t{rdh1:?}\n\tdataformat_reserved0: {dataformat_reserved0:?}\n\t{rdh2:?}\n\treserved1: {tmp_res1:?}\n\t{rdh3:?}\n\treserved2: {tmp_res2:?}",
            rdh0 = self.rdh0 , link_id = self.link_id, packet_counter = self.packet_counter, cruid_dw = tmp_cruid_dw, rdh1 = self.rdh1, dataformat_reserved0 = tmp_dataformat_reserved0, rdh2 = self.rdh2, rdh3 = self.rdh3
        )
    }
}

impl RDH for RdhCru {
    fn to_styled_row_view(&self) -> String {
        use owo_colors::OwoColorize;
        let tmp_offset = self.offset_new_packet;
        let tmp_link = self.link_id;
        let tmp_packet_cnt = self.packet_counter;
        let detector_field = self.rdh3.detector_field;
        format!(
            "{rdh0}{tmp_offset:<8}{tmp_link:<6}{tmp_packet_cnt:<10}{rdh1}{data_format:<11}{rdh2}{det_field:#x}",
            rdh0 = self.rdh0.to_styled_row_view(),
            tmp_offset = tmp_offset.white().bg_rgb::<0, 99, 0>(),
            tmp_link = tmp_link.white().bg_rgb::<0, 0, 99>(),
            tmp_packet_cnt = tmp_packet_cnt.white().bg_rgb::<0, 99, 0>(),
            rdh1 = self.rdh1.to_styled_row_view(),
            data_format = self.data_format().white().bg_rgb::<0, 0, 99>(),
            rdh2 = self.rdh2.to_styled_row_view(),
            det_field = detector_field.white().bg_rgb::<0, 0, 99>()
        )
    }
}

impl RDH_CRU for RdhCru {
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

impl SerdeRdh for RdhCru {
    #[inline]
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
        let header_text = RdhCru::rdh_header_text_with_indent_to_string(7);
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
        let rdhv6 = RdhCru {
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
        };
        assert_eq!(rdhv6.data_format(), 0);
    }

    #[test]
    fn test_rdh_v7() {
        let rdh_0 = CORRECT_RDH_CRU_V7.rdh0;

        let rdh_v7 = RdhCru {
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
        };
        assert_eq!(rdh_v7.data_format(), 2);
        assert_eq!(rdh_v7.cru_id(), 0);
        assert_eq!(rdh_v7.reserved0(), 0);
        assert_eq!(rdh_v7.payload_size(), 0);
    }

    #[test]
    fn test_print_generic() {
        let rdh_v7: RdhCru = CORRECT_RDH_CRU_V7;
        let rdh_v6: RdhCru = CORRECT_RDH_CRU_V6;
        println!("{}", RdhCru::rdh_header_text_with_indent_to_string(7));
        println!("{rdh_v7}");
        println!("{rdh_v6}");
        let v = rdh_v7.version();
        println!("{v:?}");
        print_rdh_cru_v6(&rdh_v6);
        print_rdh_cru(&rdh_v7);
        println!("{}", RdhCru::rdh_header_text_with_indent_to_string(7));
        let rdh_v7: RdhCru = CORRECT_RDH_CRU_V7;
        let rdh_v6: RdhCru = CORRECT_RDH_CRU_V6;
        print_rdh_cru(&rdh_v6);
        print_rdh_cru(&rdh_v7);
    }

    fn print_rdh_cru(rdh: &RdhCru) {
        println!("{rdh}");
    }
    fn print_rdh_cru_v6(rdh: &RdhCru) {
        println!("{rdh}");
    }

    // Test from old implementation

    #[test]
    fn test_load_rdhcruv7_from_byte_slice() {
        // Create an instace of an RDH-CRU v7
        // byte slice values taken from a valid rdh from real data
        let rdhcruv7 = RdhCru::load(
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
        let rdh_v7_from_old = RdhCru::load(&mut rdhcruv7.to_byte_slice()).unwrap();
        println!("{rdh_from_old}");
        assert_eq!(rdhcruv7, rdh_from_old);
        assert_eq!(rdhcruv7.rdh0.header_size, 0x40);
        assert_ne!(rdhcruv7, CORRECT_RDH_CRU_V7);
        assert_eq!(rdh_inferred_from_old, rdh_v7_from_old);
        dbg!(rdhcruv7);
    }
}
