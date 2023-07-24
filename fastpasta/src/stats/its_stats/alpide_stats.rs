//! Contains the possible ALPIDE stats that can be collected during analysis

/// Struct to store observed ALPIDE stats
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct AlpideStats {
    readout_flags: ReadoutFlags,
}

impl AlpideStats {
    pub(crate) fn log_readout_flags(&mut self, chip_trailer: u8) {
        self.readout_flags.log(chip_trailer);
    }

    pub(crate) fn readout_flags(&self) -> ReadoutFlags {
        self.readout_flags
    }

    pub(crate) fn sum(&mut self, other: AlpideStats) {
        self.readout_flags = self.readout_flags.sum(other.readout_flags);
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub(crate) struct ReadoutFlags {
    pub(super) chip_trailers_seen: u32,
    pub(super) busy_violations: u32,       // 4'b1000
    pub(super) data_overrun: u32,          // 4'b1100
    pub(super) transmission_in_fatal: u32, // 4'b1110
    pub(super) flushed_incomplete: u32,    // 4'bx1xx
    pub(super) strobe_extended: u32,       // 4'bxx1x
    pub(super) busy_transitions: u32,      // 4'bxxx1
}

impl ReadoutFlags {
    const CHIP_TRAILER_BUSY_VIOLATION: u8 = 0b1011_1000;
    const CHIP_TRAILER_DATA_OVERRUN: u8 = 0b1011_1100;
    const CHIP_TRAILER_TRANSMISSION_IN_FATAL: u8 = 0b1011_1110;
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

    pub fn chip_trailers_seen(&self) -> u32 {
        self.chip_trailers_seen
    }
    pub fn busy_violations(&self) -> u32 {
        self.busy_violations
    }
    pub fn flushed_incomplete(&self) -> u32 {
        self.flushed_incomplete
    }
    pub fn strobe_extended(&self) -> u32 {
        self.strobe_extended
    }
    pub fn busy_transitions(&self) -> u32 {
        self.busy_transitions
    }
    pub fn data_overrun(&self) -> u32 {
        self.data_overrun
    }
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
}
