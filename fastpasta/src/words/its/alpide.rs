#![allow(dead_code)]

//! Word definitions and utility functions for working with ALPIDE data words

pub mod alpide_word;

/// Contains information from a single ALPIDE chip in a single frame
///
/// Unsafe/Invalid if used outside of the context of a single frame
pub struct AlpideFrameChipData {
    /// The ID of the chip the data is from
    pub(crate) chip_id: u8,
    /// Bunch counter for the frame \[10:3\]
    pub(crate) bunch_counter: Option<u8>,
    /// Other data from the chip
    pub(crate) data: Vec<u8>,
}

impl AlpideFrameChipData {
    /// Create a new instance from the chip ID
    pub fn from_id(chip_id: u8) -> Self {
        Self {
            chip_id,
            bunch_counter: None,
            data: Vec::new(),
        }
    }
    /// Create a new instance from the chip ID but disallow adding data
    ///
    /// A light weight version of `from_id` that is used when the data is not needed
    pub fn from_id_no_data(chip_id: u8) -> Self {
        Self {
            chip_id,
            bunch_counter: None,
            data: Vec::with_capacity(0),
        }
    }

    /// Store the bunch counter for a chip in a frame
    ///
    /// If the bunch counter has already been set, an error is returned describing the Chip ID,
    /// the current bunch counter, and the bunch counter that was attempted to be set
    pub fn store_bc(&mut self, bc: u8) -> Result<(), String> {
        if self.bunch_counter.is_some() {
            return Err(format!(
                "Bunch counter already set for chip {id}, is {current_bc}, tried to set to {new_bc}",
                id = self.chip_id,
                current_bc = self.bunch_counter.unwrap(),
                new_bc = bc
            ));
        }
        self.bunch_counter = Some(bc);
        Ok(())
    }
}
