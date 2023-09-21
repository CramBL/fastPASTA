//! Contains the possible ALPIDE stats that can be collected during analysis

use serde::{Deserialize, Serialize};

/// Struct to store observed ALPIDE stats
#[derive(Default, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlpideStats {
    readout_flags: ReadoutFlags,
}

impl AlpideStats {
    pub(crate) fn log_readout_flags(&mut self, chip_trailer: u8) {
        self.readout_flags.log(chip_trailer);
    }

    /// Returns a reference to the readout flags
    pub fn readout_flags(&self) -> &ReadoutFlags {
        &self.readout_flags
    }

    pub(crate) fn sum(&mut self, other: AlpideStats) {
        self.readout_flags = self.readout_flags.sum(other.readout_flags);
    }

    pub(crate) fn validate_other(&self, other: &Self) -> Result<(), Vec<String>> {
        let mut errs: Vec<String> = vec![];

        if let Err(mut sub_errs) = self.readout_flags.validate_other(&other.readout_flags) {
            errs.append(&mut sub_errs);
        }

        // Do this (syntax) to ensure that adding a new field to the struct doesn't break the validation
        // If a new field is added, this will fail to compile, before explicitly adding the new field to this instantiation
        // unused right now as there's only a sub struct which is validated above
        let _other = Self {
            readout_flags: ReadoutFlags::default(),
        };

        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }

        //self.validate_fields(&other)
    }
    // Implementation of the `validate_fields` macro
    // Remember to add new fields here as well!
    // Commented out as there's only a sub struct as of now
    //crate::validate_fields!(AlpideStats, readout_flags);
}

/// Struct to store the readout flags observed in ALPIDE chip trailers
#[derive(Default, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadoutFlags {
    chip_trailers_seen: u32,
    busy_violations: u32,       // 4'b1000
    data_overrun: u32,          // 4'b1100
    transmission_in_fatal: u32, // 4'b1110
    flushed_incomplete: u32,    // 4'bx1xx
    strobe_extended: u32,       // 4'bxx1x
    busy_transitions: u32,      // 4'bxxx1
}

impl ReadoutFlags {
    const CHIP_TRAILER_BUSY_VIOLATION: u8 = 0b1011_1000;
    const CHIP_TRAILER_DATA_OVERRUN: u8 = 0b1011_1100;
    const CHIP_TRAILER_TRANSMISSION_IN_FATAL: u8 = 0b1011_1110;

    /// Log a chip trailer and update the stats
    pub fn log(&mut self, chip_trailer: u8) {
        self.chip_trailers_seen += 1;
        if chip_trailer == Self::CHIP_TRAILER_BUSY_VIOLATION {
            self.busy_violations += 1;
            return; // The other flags are not set in this case
        } else if chip_trailer == Self::CHIP_TRAILER_DATA_OVERRUN {
            self.data_overrun += 1;
            return;
        } else if chip_trailer == Self::CHIP_TRAILER_TRANSMISSION_IN_FATAL {
            self.transmission_in_fatal += 1;
            return;
        }
        if chip_trailer & 0b0000_0100 == 0b0000_0100 {
            self.flushed_incomplete += 1;
        }
        if chip_trailer & 0b0000_0010 == 0b0000_0010 {
            self.strobe_extended += 1;
        }
        if chip_trailer & 0b0000_0001 == 0b0000_0001 {
            self.busy_transitions += 1;
        }
    }

    /// Returns the number of chip trailers seen
    pub fn chip_trailers_seen(&self) -> u32 {
        self.chip_trailers_seen
    }

    /// Returns the number of chip trailers with busy violations
    pub fn busy_violations(&self) -> u32 {
        self.busy_violations
    }

    /// Returns the number of chip trailers with data overrun
    pub fn flushed_incomplete(&self) -> u32 {
        self.flushed_incomplete
    }

    /// Returns the number of chip trailers with strobe extended
    pub fn strobe_extended(&self) -> u32 {
        self.strobe_extended
    }

    /// Returns the number of chip trailers with busy transitions
    pub fn busy_transitions(&self) -> u32 {
        self.busy_transitions
    }

    /// Returns the number of chip trailers with data overrun
    pub fn data_overrun(&self) -> u32 {
        self.data_overrun
    }

    /// Returns the number of chip trailers with transmission in fatal
    pub fn transmission_in_fatal(&self) -> u32 {
        self.transmission_in_fatal
    }

    fn sum(self, other: ReadoutFlags) -> Self {
        Self {
            chip_trailers_seen: self.chip_trailers_seen + other.chip_trailers_seen,
            busy_violations: self.busy_violations + other.busy_violations,
            flushed_incomplete: self.flushed_incomplete + other.flushed_incomplete,
            strobe_extended: self.strobe_extended + other.strobe_extended,
            busy_transitions: self.busy_transitions + other.busy_transitions,
            data_overrun: self.data_overrun + other.data_overrun,
            transmission_in_fatal: self.transmission_in_fatal + other.transmission_in_fatal,
        }
    }

    pub(super) fn validate_other(&self, other: &Self) -> Result<(), Vec<String>> {
        // Do this (syntax) to ensure that adding a new field to the struct doesn't break the validation
        // If a new field is added, this will fail to compile, before explicitly adding the new field to this instantiation
        // Remebmer to add new fields to the `validate_fields` macro as well!
        let other = Self {
            chip_trailers_seen: other.chip_trailers_seen,
            busy_violations: other.busy_violations,
            flushed_incomplete: other.flushed_incomplete,
            strobe_extended: other.strobe_extended,
            busy_transitions: other.busy_transitions,
            data_overrun: other.data_overrun,
            transmission_in_fatal: other.transmission_in_fatal,
        };

        self.validate_fields(&other)
    }

    // Implementation of the `validate_fields` macro
    // Remember to add new fields here as well!
    crate::validate_fields!(
        ReadoutFlags,
        chip_trailers_seen,
        busy_violations,
        flushed_incomplete,
        strobe_extended,
        busy_transitions,
        data_overrun,
        transmission_in_fatal
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_serde_consistency() {
        // Test JSON and TOML serialization/deserialization
        let mut alpide_stats = AlpideStats::default();
        alpide_stats.log_readout_flags(ReadoutFlags::CHIP_TRAILER_BUSY_VIOLATION);

        // JSON
        let alpide_stats_ser_json = serde_json::to_string(&alpide_stats).unwrap();
        let alpide_stats_de_json: AlpideStats =
            serde_json::from_str(&alpide_stats_ser_json).unwrap();
        assert_eq!(alpide_stats, alpide_stats_de_json);
        println!("{}", serde_json::to_string_pretty(&alpide_stats).unwrap());

        // TOML
        let alpide_stats_ser_toml = toml::to_string(&alpide_stats).unwrap();
        let alpide_stats_de_toml: AlpideStats = toml::from_str(&alpide_stats_ser_toml).unwrap();
        assert_eq!(alpide_stats, alpide_stats_de_toml);
        println!("{}", alpide_stats_ser_toml);
    }
}
