//! Contains the [LaneAlpideFrameAnalyzer] struct that analyzes the data from a single lane in a readout frame from the ALPIDE chips.
//!
//! Analysis consists of decoding the ALPIDE data and then performing checks on the decoded data.

use std::hint::unreachable_unchecked;

use crate::{
    stats::its_stats::alpide_stats::AlpideStats,
    words::its::{
        alpide_words::{AlpideFrameChipData, LaneDataFrame},
        Layer,
    },
};
use itertools::Itertools;

/// Decodes the ALPIDE data from a readout frame for a single lane
pub struct LaneAlpideFrameAnalyzer<'a> {
    // Works on a single lane at a time
    lane_number: u8,
    is_header_seen: bool, // Set when a Chip Header is seen, reset when a Chip Trailer is seen
    last_chip_id: u8,     // 4 bits
    skip_n_bytes: u8, // Used when an irrelevant word larger than 1 byte is seen, to skip the next n bytes
    chip_data: Vec<AlpideFrameChipData>,
    // Indicate that the next byte should be saved as bunch counter for frame
    next_is_bc: bool,
    // Indicates that the lane status SHOULD be fatal. In this case only padding bytes should be observed which would have no effect the rest of analysis.
    // Meaning that decoding will continue until the end of the frame, but no checks will be performed.
    lane_status_fatal: bool,
    errors: Option<String>,
    from_layer: Option<Layer>,
    validated_bc: Option<u8>, // Bunch counter for the frame if the bunch counters match
    valid_chip_order_ob: Option<&'a [Vec<u8>]>, // Valid chip orders for Outer Barrel
    valid_chip_count_ob: Option<u8>, // Valid chip count for Outer Barrel
    /// Stats about the ALPIDE data
    alpide_stats: AlpideStats,
}

// impl for core utility
impl<'a> LaneAlpideFrameAnalyzer<'a> {
    const ERR_MSG_PREFIX: &'static str = "\n\t\t\t"; // Newline + indentation for error messages
    const IL_CHIP_COUNT: usize = 1; // Number of chips in an inner layer readout frame
    const ML_OL_CHIP_COUNT: usize = 7; // Number of chips in a middle/outer layer readout frame

    /// Creates a new decoder by specifying the layer the data is from
    pub fn new(
        data_origin: Layer,
        valid_chip_order_ob: Option<&'a [Vec<u8>]>,
        valid_chip_count_ob: Option<u8>,
    ) -> Self {
        Self {
            lane_number: 0,
            is_header_seen: false,
            last_chip_id: 0,
            skip_n_bytes: 0,
            chip_data: match data_origin {
                // ALPIDE data from IB should have 9 chips per frame, OB should have 7
                Layer::Inner => Vec::with_capacity(Self::IL_CHIP_COUNT),
                Layer::Middle | Layer::Outer => Vec::with_capacity(Self::ML_OL_CHIP_COUNT),
            },
            next_is_bc: false,
            lane_status_fatal: false,
            errors: Some(String::new()),
            from_layer: Some(data_origin),
            validated_bc: None,
            valid_chip_order_ob,
            valid_chip_count_ob,
            alpide_stats: AlpideStats::default(),
        }
    }

    /// Decodes the readout frame for a lane byte by byte, then performs checks on the data and stores error messages
    ///
    /// First data is decoded, then it is validated.
    /// If the validation fails, the error messages are stored in the errors vector that is returned.
    pub fn analyze_alpide_frame(&mut self, lane_data_frame: &LaneDataFrame) -> Result<(), String> {
        self.lane_number = lane_data_frame.lane_number(self.from_layer.unwrap());
        log::debug!(
            "Processing ALPIDE frame for lane {lane_id}",
            lane_id = lane_data_frame.lane_id
        );
        lane_data_frame.lane_data.iter().for_each(|alpide_byte| {
            self.decode(*alpide_byte);
        });
        if self.lane_status_fatal {
            // If the lane status is fatal, skip the rest of the analysis
            Ok(())
        } else {
            self.do_lane_alpide_checks()
        }
    }

    /// Takes one ALPIDE byte at a time and decodes information from it.
    fn decode(&mut self, alpide_byte: u8) {
        use crate::words::its::alpide_words::{AlpideProtocolExtension, AlpideWord};
        log::trace!("Processing {alpide_byte:#02X} ALPIDE byte");

        if self.skip_n_bytes > 0 {
            self.skip_n_bytes -= 1;
            return;
        }
        if self.next_is_bc {
            if let Err(msg) = self.store_bunch_counter(alpide_byte) {
                self.errors.as_mut().unwrap().push_str(&msg);
            }

            // Done with the byte containing the bunch counter
            self.next_is_bc = false;

            // Skip to next byte
            return;
        }

        if !self.is_header_seen && alpide_byte == 0 {
            return; // Padding byte
        }

        match AlpideWord::from_byte(alpide_byte) {
            Ok(word) => {
                match word {
                    AlpideWord::ChipHeader => {
                        self.is_header_seen = true;
                        self.last_chip_id = alpide_byte & 0b1111;
                        self.next_is_bc = true;
                        log::trace!("{alpide_byte:#02X}: ChipHeader");
                    }
                    AlpideWord::ChipEmptyFrame => {
                        self.is_header_seen = false;
                        self.last_chip_id = alpide_byte & 0b1111;
                        self.next_is_bc = true;
                        log::trace!("{alpide_byte:#02X}: ChipEmptyFrame");
                    }
                    AlpideWord::ChipTrailer => {
                        self.is_header_seen = false;
                        self.alpide_stats.log_readout_flags(alpide_byte);
                        log::trace!("{alpide_byte:#02X}: ChipTrailer");
                    } // Reset the header seen flag
                    AlpideWord::RegionHeader => {
                        self.is_header_seen = true;
                        log::trace!("{alpide_byte:#02X}: RegionHeader");
                    } // Do nothing at the moment
                    AlpideWord::DataShort => {
                        self.skip_n_bytes = 1;
                        log::trace!("{alpide_byte:#02X}: DataShort");
                    } // Skip the next byte
                    AlpideWord::DataLong => {
                        self.skip_n_bytes = 2;
                        log::trace!("{alpide_byte:#02X}: DataLong");
                    } // Skip the next 2 bytes
                    AlpideWord::BusyOn => log::trace!("{alpide_byte:#02X}: BusyOn word seen!"),
                    AlpideWord::BusyOff => log::trace!("{alpide_byte:#02X}: BusyOff word seen!"),
                    AlpideWord::Ape(ape) => match ape {
                        // Lane status = WARNING
                        AlpideProtocolExtension::StripStart => {
                            log::warn!("{alpide_byte:#02X}: APE_STRIP_START seen!")
                        }
                        AlpideProtocolExtension::PeDataMissing => {
                            log::warn!("{alpide_byte:#02X}: APE_PE_DATA_MISSING seen!")
                        }
                        AlpideProtocolExtension::OotDataMissing => {
                            log::warn!("{alpide_byte:#02X}: APE_OOT_DATA_MISSING seen!")
                        }
                        // Unreachable because the earlier check !is_header_seen && alpide_byte == 0 maches padding bytes
                        // And in this match statement, a padding byte would instead be interpreted as a Data Long
                        AlpideProtocolExtension::Padding => unsafe { unreachable_unchecked() },
                        // APEs signifying Lane status = FATAL
                        fatal_ape => {
                            log::warn!("{alpide_byte:#02X}: {APE} seen! This APE indicates FATAL lane status!", APE = fatal_ape);
                            self.lane_status_fatal = true;
                        }
                    },
                }
            }
            Err(_) => {
                log::warn!("Unknown ALPIDE word: {alpide_byte:#02X}")
            }
        }
    }

    // All checks performed after decoding starts here
    fn do_lane_alpide_checks(&mut self) -> Result<(), String> {
        // Check all bunch counters match
        if let Err(msg) = self.check_bunch_counters() {
            // if it is already in the errors_per_lane, add it to the list
            self.errors
                .as_mut()
                .unwrap()
                .push_str(&format!("\n\t\t [E9003] Chip bunch counter mismatch:{msg}"));
        }

        if let Err(msg) = self.check_chip_count() {
            self.errors
                .as_mut()
                .unwrap()
                .push_str(&format!("\n\t\t [E9004] Chip ID count mismatch:{msg}"));
        } else {
            // Only check if the chip count is valid.
            // Check chip ID order
            if let Err(msg) = self.check_chip_id_order() {
                self.errors
                    .as_mut()
                    .unwrap()
                    .push_str(&format!("\n\t\t [E9005] Chip ID order mismatch:{msg}"))
            }
        }

        if self.has_errors() {
            Err(self.errors.take().unwrap())
        } else {
            Ok(())
        }
    }

    /// Check that all bunch counters are identical
    ///
    /// If the check passes, the bunch counter value is stored as the validated bunch counter (bc).
    fn check_bunch_counters(&mut self) -> Result<(), String> {
        // Return all unique bunch counters
        let unique_bcs: Vec<&AlpideFrameChipData> = self
            .chip_data
            .iter()
            .unique_by(|cd| cd.bunch_counter)
            .collect_vec();
        // If there is more than one unique bunch counter (this should not happen)
        if unique_bcs.len() > 1 {
            // Count which bunch counters are found for which chip IDs
            let mut bc_to_chip_ids: Vec<(u8, Vec<u8>)> = Vec::new();
            unique_bcs.iter().for_each(|chip| {
                // Iterate through each unique bunch counter
                if let Some(bc) = chip.bunch_counter {
                    // Collect all chip IDs that have the same bunch counter
                    let mut bc_to_chip_id: (u8, Vec<u8>) = (bc, Vec::new());
                    // Iterate through each chip ID and compare the bunch counter
                    self.chip_data.iter().for_each(|cd| {
                        // If the bunch counter matches, add the chip ID to the vector
                        if bc == cd.bunch_counter.unwrap() {
                            bc_to_chip_id.1.push(cd.chip_id);
                        }
                    });
                    // Add the bunch counter and the chip IDs to the vector
                    bc_to_chip_ids.push(bc_to_chip_id);
                }
            });
            // Print the bunch counters and the chip IDs that have the same bunch counter
            let error_str = bc_to_chip_ids
                .iter()
                .fold(String::from(""), |acc, (bc, chip_ids)| {
                    format!(
                        "{acc}{newline_indent}Bunch counter: {bc:>3?} | Chip IDs: {chip_ids:?}",
                        newline_indent = Self::ERR_MSG_PREFIX
                    )
                });
            Err(error_str)
        } else {
            self.validated_bc = unique_bcs.first().unwrap().bunch_counter;
            Ok(())
        }
    }

    fn check_chip_count(&self) -> Result<(), String> {
        // Check if the number of chip data matches the expected number of chips
        if matches!(self.from_layer, Some(Layer::Inner)) {
            if self.chip_data.len() != Self::IL_CHIP_COUNT {
                return Err(format!(
                    "{newline_indent}Expected {expected_chip_count} Chip ID in IB but found {id_cnt}: {chip_ids:?}",
                    expected_chip_count = Self::IL_CHIP_COUNT,
                    newline_indent = Self::ERR_MSG_PREFIX,
                    id_cnt = self.chip_data.len(),
                    chip_ids = self.chip_data.iter().map(|cd| cd.chip_id).collect_vec()
                ));
            }
        }
        // Middle or Outer layer (Outer barrel)
        else if let Some(custom_chip_count_check) = self.valid_chip_count_ob {
            if self.chip_data.len() != custom_chip_count_check as usize {
                return Err(format!(
                    "{newline_indent}Expected {expected_chip_count} Chip ID(s) in OB but found {id_cnt}: {chip_ids:?}",
                    expected_chip_count = custom_chip_count_check,
                    newline_indent = Self::ERR_MSG_PREFIX,
                    id_cnt = self.chip_data.len(),
                    chip_ids = self.chip_data.iter().map(|cd| cd.chip_id).collect_vec()
                ));
            }
        }
        Ok(())
    }

    fn check_chip_id_order(&self) -> Result<(), String> {
        // Get the chip IDs from the chip data vector
        let chip_ids: Vec<u8> = self.chip_data.iter().map(|cd| cd.chip_id).collect();
        if let Some(data_from) = &self.from_layer {
            match data_from {
                Layer::Inner => {
                    // IB only has one chip but it should match the lane number
                    if chip_ids[0] != self.lane_number {
                        return Err(format!(
                            "{newline_indent}Expected Chip ID {lane} in IB but found {chip_id}",
                            newline_indent = Self::ERR_MSG_PREFIX,
                            lane = self.lane_number,
                            chip_id = chip_ids[0]
                        ));
                    }
                }
                Layer::Middle | Layer::Outer => {
                    // Check that the chip IDs are in the correct order
                    if let Some(valid_orderings) = self.valid_chip_order_ob {
                        if !valid_orderings.contains(&chip_ids) {
                            // If the chip IDs do not match any of the valid orders, return an error
                            return Err(format!(
                                    "{newline_indent}Expected any order={valid_orderings:?} in {layer} but found {chip_ids:?}",
                                    newline_indent = Self::ERR_MSG_PREFIX,
                                    layer = data_from,
                                    chip_ids = chip_ids
                                ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn store_bunch_counter(&mut self, bc: u8) -> Result<(), String> {
        // Search for the chip data matching the last chip id
        if let Some(cd) = self
            .chip_data
            .iter_mut()
            .find(|cd| cd.chip_id == self.last_chip_id)
        {
            // Store the bunch counter for the chip data
            cd.store_bc(bc)?;
        } else {
            // ID not found, create a instance of AlpideFrameChipData with the ID
            let mut cd = AlpideFrameChipData::from_id_no_data(self.last_chip_id);
            // Add the bunch counter to the bunch counter vector
            cd.store_bc(bc)?;
            // Add the chip data to the chip data vector
            self.chip_data.push(cd);
        }

        Ok(())
    }
}

// impl for utility member functions
impl<'a> LaneAlpideFrameAnalyzer<'a> {
    /// Print the bunch counter for each chip
    pub fn print_chip_bunch_counters(&self) {
        self.chip_data
            .iter()
            .sorted_unstable_by(|a, b| Ord::cmp(&a.chip_id, &b.chip_id))
            .for_each(|cd| {
                println!(
                    "Chip ID: {:>2} | Bunch counter: {:?}",
                    cd.chip_id,
                    cd.bunch_counter.unwrap()
                );
            });
    }

    fn has_errors(&self) -> bool {
        if let Some(error_msg) = self.errors.as_ref() {
            !error_msg.is_empty()
        } else {
            false
        }
    }

    /// Get if the lane status is fatal (To avoid checking the data against other lanes that were validated in the same readout frame)
    pub fn is_fatal_lane(&self) -> bool {
        self.lane_status_fatal
    }

    /// Get the validated bunch counter. Is `None` if the bunch counters are not identical.
    pub fn validated_bc(&self) -> Option<u8> {
        self.validated_bc
    }

    /// Get the collected ALPIDE stats
    pub fn alpide_stats(&mut self) -> &AlpideStats {
        &self.alpide_stats
    }
}
