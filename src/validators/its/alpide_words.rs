#![allow(dead_code)]
use crate::words::its::alpide_words::AlpideFrameChipData;
use itertools::Itertools;

#[derive(Default)]
pub struct AlpideFrameDecoder {
    is_header_seen: bool, // Set when a Chip Header is seen, reset when a Chip Trailer is seen
    last_chip_id: u8,     // 4 bits
    last_region_id: u8,   // 5 bits
    skip_n_bytes: u8, // Used when an irrelevant word larger than 1 byte is seen, to skip the next n bytes
    chip_data: Vec<AlpideFrameChipData>,
    // Indicate that the next byte should be saved as bunch counter for frame
    next_is_bc: bool,
    warning_count: u8,
    errors: Vec<String>,
}

impl AlpideFrameDecoder {
    pub fn process(&mut self, alpide_bytes: &[u8]) {
        use crate::words::its::alpide_words::AlpideWord;
        self.warning_count = 0; // Reset warnings
        for (i, b) in alpide_bytes.iter().enumerate() {
            if self.skip_n_bytes > 0 {
                self.skip_n_bytes -= 1;
                continue;
            }
            if self.next_is_bc {
                if let Err(msg) = self.store_bunch_counter(*b) {
                    self.warning_count += 1;
                    self.errors.push(msg);
                }

                // Done with the byte containing the bunch counter
                self.next_is_bc = false;

                // Skip to next byte
                continue;
            }

            if !self.is_header_seen && *b == 0 {
                continue; // IDLE word
            }

            match AlpideWord::from_byte(*b) {
                Ok(word) => match word {
                    AlpideWord::ChipHeader => {
                        self.is_header_seen = true;
                        let chip_id = *b & 0b1111;
                        self.last_chip_id = chip_id;
                        self.next_is_bc = true;
                    }
                    AlpideWord::ChipEmptyFrame => {
                        self.is_header_seen = false;
                        let chip_id = *b & 0b1111;
                        self.last_chip_id = chip_id;
                        self.next_is_bc = true;
                    }
                    AlpideWord::ChipTrailer => self.is_header_seen = false, // Reset the header seen flag
                    AlpideWord::RegionHeader => (), // Do nothing at the moment
                    AlpideWord::DataShort => self.skip_n_bytes = 1, // Skip the next byte
                    AlpideWord::DataLong => self.skip_n_bytes = 2, // Skip the next 2 bytes
                    AlpideWord::BusyOn => log::info!("BusyOn word seen!"),
                    AlpideWord::BusyOff => log::info!("BusyOff word seen!"),
                },
                Err(_) => {
                    self.warning_count += 1;
                    log::warn!("Unknown ALPIDE word: {:#02X} at index {i}", b)
                }
            }
        }
    }

    fn store_bunch_counter(&mut self, bc: u8) -> Result<(), String> {
        // Search for the chip data matching the last chip id
        match self
            .chip_data
            .iter_mut()
            .find(|cd| cd.chip_id == self.last_chip_id)
        {
            Some(cd) => {
                // Store the bunch counter for the chip data
                cd.store_bc(bc)?;
            }
            None => {
                // ID not found, create a instance of AlpideFrameChipData with the ID
                let mut cd = AlpideFrameChipData::from_id_no_data(self.last_chip_id);
                // Add the bunch counter to the bunch counter vector
                cd.store_bc(bc)?;
                // Add the chip data to the chip data vector
                self.chip_data.push(cd);
            }
        }
        Ok(())
    }

    pub fn print_chip_bunch_counters(&self) {
        self.chip_data
            .iter()
            .sorted_by(|a, b| Ord::cmp(&a.chip_id, &b.chip_id))
            .for_each(|cd| {
                println!(
                    "Chip ID: {:>2} | Bunch counter: {:?}",
                    cd.chip_id,
                    cd.bunch_counter.unwrap()
                );
            });
    }

    pub fn check_bunch_counters(&self) -> Result<(), String> {
        // Return all unique bunch counters
        let unique_bcs = self
            .chip_data
            .iter()
            .unique_by(|cd| cd.bunch_counter)
            .collect_vec();
        // If there is more than one unique bunch counter (this should not happen)
        if unique_bcs.len() > 1 {
            // Count which bunch counters are found for which chip IDs
            let mut bc_to_chip_ids: Vec<(u8, Vec<u8>)> = Vec::new();
            unique_bcs.iter().for_each(|cd| {
                // Iterate through each unique bunch counter
                if let Some(bc) = cd.bunch_counter {
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
            log::warn!("Multiple different bunch counters found in ALPIDE Data Frame!");
            // Print the bunch counters and the chip IDs that have the same bunch counter
            let error_str = bc_to_chip_ids
                .iter()
                .fold(String::from(""), |acc, (bc, chip_ids)| {
                    format!("{acc}\n\t\tBunch counter: {bc:?} | Chip IDs: {chip_ids:?}")
                });
            Err(error_str)
        } else {
            Ok(())
        }
    }

    pub fn has_warnings(&self) -> bool {
        !self.warning_count == 0
    }

    pub fn warning_count(&self) -> u8 {
        self.warning_count
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn consume_errors(&mut self) -> std::vec::Drain<String> {
        self.errors.drain(..)
    }
}
