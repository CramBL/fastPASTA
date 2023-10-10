//! Contains test data for testing functionality on a [RDH CRU][RdhCru].

use super::rdh0::*;
use super::rdh1::*;
use super::rdh2::*;
use super::rdh3::*;
use super::rdh_cru::*;

// For testing
/// Convenience struct of a [RDH CRU][RdhCru] version 7 used in tests.
pub const CORRECT_RDH_CRU_V7: RdhCru = RdhCru {
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
        // SOC
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

/// Convenience struct of a [RDH CRU][RdhCru] version 7 used in tests.
pub const CORRECT_RDH_CRU_V7_SOT: RdhCru = RdhCru {
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
        // SOT
        trigger_type: 0b0100_1000_1001_0011,
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

/// Convenience struct of a [RDH CRU][RdhCru] version 6 used in tests.
pub const CORRECT_RDH_CRU_V6: RdhCru = RdhCru {
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
};

/// Convenience struct of an [RDH CRU][RdhCru] coming after an initial [RDH CRU][RdhCru] with the version [V7] used in tests.
pub const CORRECT_RDH_CRU_V7_NEXT: RdhCru = RdhCru {
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
};

/// Convenience struct of an [RDH CRU][RdhCru] closing an HBF, used in tests.
pub const CORRECT_RDH_CRU_V7_NEXT_NEXT_STOP: RdhCru = RdhCru {
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
};
