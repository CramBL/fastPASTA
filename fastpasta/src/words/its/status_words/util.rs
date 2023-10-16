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

/// Checks if the corrosponding lane bit is set in the IHW active lanes field
pub fn is_lane_active(lane: u8, active_lanes: u32) -> bool {
    //log::debug!("Lane: {lane}, Active lanes: {active_lanes:#X}");
    let lane = lane as u32;
    let mask = 1 << lane;
    active_lanes & mask != 0
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

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
