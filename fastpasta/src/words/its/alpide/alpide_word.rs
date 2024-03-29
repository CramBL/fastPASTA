//! ALPIDE words and APEs
#![allow(dead_code)]
use crate::util::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AlpideProtocolExtension {
    /// Padding word - Lane status = OK
    Padding,
    /// Strip start - Lane status = WARNING
    StripStart,
    /// Detector timeout - Lane status = FATAL
    DetectorTimeout,
    /// Out of table (8b10b OOT) - Lane status = FATAL
    OutOfTable,
    /// Protocol error - Lane status = FATAL
    ProtocolError,
    /// Lane FIFO overflow error - Lane status = FATAL
    LaneFifoOverflowError,
    /// FSM error - Lane status = FATAL
    FsmError,
    /// Pending detector event limit - Lane status = FATAL
    PendingDetectorEventLimit,
    /// Pending lane event limit - Lane status = FATAL
    PendingLaneEventLimit,
    /// O2N error - Lane status = FATAL
    O2nError,
    /// Rate missing trigger error - Lane status = FATAL
    RateMissingTriggerError,
    /// PE data missing - Lane status = WARNING
    PeDataMissing,
    /// OOT data missing - Lane status = WARNING
    OotDataMissing,
}

impl AlpideProtocolExtension {
    // ALPIDE Protocol Extension (APE) words
    const APE_PADDING: u8 = 0x00;
    const APE_STRIP_START: u8 = 0xF2; // Lane status = WARNING
    const APE_DET_TIMEOUT: u8 = 0xF4; // Lane status = FATAL
    const APE_OOT: u8 = 0xF5; // Lane status = FATAL
    const APE_PROTOCOL_ERROR: u8 = 0xF6; // Lane status = FATAL
    const APE_LANE_FIFO_OVERFLOW_ERROR: u8 = 0xF7; // Lane status = FATAL
    const APE_FSM_ERROR: u8 = 0xF8; // Lane status = FATAL
    const APE_PENDING_DETECTOR_EVENT_LIMIT: u8 = 0xF9; // Lane status = FATAL
    const APE_PENDING_LANE_EVENT_LIMIT: u8 = 0xFA; // Lane status = FATAL
    const APE_O2N_ERROR: u8 = 0xFB; // Lane status = FATAL
    const APE_RATE_MISSING_TRG_ERROR: u8 = 0xFC; // Lane status = FATAL
    const APE_PE_DATA_MISSING: u8 = 0xFD; // Lane status = WARNING
    const APE_OOT_DATA_MISSING: u8 = 0xFE; // Lane status = WARNING

    #[inline]
    fn from_byte(b: u8) -> Result<AlpideWord, ()> {
        match b {
            Self::APE_STRIP_START => Ok(AlpideWord::Ape(AlpideProtocolExtension::StripStart)),
            Self::APE_DET_TIMEOUT => Ok(AlpideWord::Ape(AlpideProtocolExtension::DetectorTimeout)),
            Self::APE_OOT => Ok(AlpideWord::Ape(AlpideProtocolExtension::OutOfTable)),
            Self::APE_PROTOCOL_ERROR => Ok(AlpideWord::Ape(AlpideProtocolExtension::ProtocolError)),
            Self::APE_LANE_FIFO_OVERFLOW_ERROR => Ok(AlpideWord::Ape(
                AlpideProtocolExtension::LaneFifoOverflowError,
            )),
            Self::APE_FSM_ERROR => Ok(AlpideWord::Ape(AlpideProtocolExtension::FsmError)),
            Self::APE_PENDING_DETECTOR_EVENT_LIMIT => Ok(AlpideWord::Ape(
                AlpideProtocolExtension::PendingDetectorEventLimit,
            )),
            Self::APE_PENDING_LANE_EVENT_LIMIT => Ok(AlpideWord::Ape(
                AlpideProtocolExtension::PendingLaneEventLimit,
            )),
            Self::APE_O2N_ERROR => Ok(AlpideWord::Ape(AlpideProtocolExtension::O2nError)),
            Self::APE_RATE_MISSING_TRG_ERROR => Ok(AlpideWord::Ape(
                AlpideProtocolExtension::RateMissingTriggerError,
            )),
            Self::APE_PE_DATA_MISSING => {
                Ok(AlpideWord::Ape(AlpideProtocolExtension::PeDataMissing))
            }
            Self::APE_OOT_DATA_MISSING => {
                Ok(AlpideWord::Ape(AlpideProtocolExtension::OotDataMissing))
            }
            Self::APE_PADDING => Ok(AlpideWord::Ape(AlpideProtocolExtension::Padding)),
            _ => Err(()),
        }
    }
}

impl fmt::Display for AlpideProtocolExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlpideProtocolExtension::Padding => write!(f, "APE_PADDING"),
            AlpideProtocolExtension::StripStart => write!(f, "APE_STRIP_START"),
            AlpideProtocolExtension::DetectorTimeout => write!(f, "APE_DET_TIMEOUT"),
            AlpideProtocolExtension::OutOfTable => write!(f, "APE_OOT"),
            AlpideProtocolExtension::ProtocolError => write!(f, "APE_PROTOCOL_ERROR"),
            AlpideProtocolExtension::LaneFifoOverflowError => {
                write!(f, "APE_LANE_FIFO_OVERFLOW_ERROR")
            }
            AlpideProtocolExtension::FsmError => write!(f, "APE_FSM_ERROR"),
            AlpideProtocolExtension::PendingDetectorEventLimit => {
                write!(f, "APE_PENDING_DETECTOR_EVENT_LIMIT")
            }
            AlpideProtocolExtension::PendingLaneEventLimit => {
                write!(f, "APE_PENDING_LANE_EVENT_LIMIT")
            }
            AlpideProtocolExtension::O2nError => write!(f, "APE_O2N_ERROR"),
            AlpideProtocolExtension::RateMissingTriggerError => {
                write!(f, "APE_RATE_MISSING_TRG_ERROR")
            }
            AlpideProtocolExtension::PeDataMissing => write!(f, "APE_PE_DATA_MISSING"),
            AlpideProtocolExtension::OotDataMissing => write!(f, "APE_OOT_DATA_MISSING"),
        }
    }
}

/// All the possible words that can be found in the ALPIDE data stream
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AlpideWord {
    ChipHeader,     // 1010<chip id[3:0]><BUNCH_COUNTER_FOR_FRAME[10:3]>
    ChipEmptyFrame, // 1110<chip id[3:0]><BUNCH COUNTER FOR FRAME[10:3]>
    ChipTrailer,    // 1011<readout flags[3:0]>
    RegionHeader,   // 110<region id[4:0]>
    DataShort,      // 01<encoder id[3:0]><addr[9:0]>
    DataLong,       // 00<encoder id[3:0]><addr[9:0]>0<hit map[6:0]>
    BusyOn,         // 1111_0001
    BusyOff,        // 1111_0000
    Ape(AlpideProtocolExtension),
}

impl AlpideWord {
    const CHIP_HEADER: u8 = 0xA0; // 1010_<chip_id[3:0]> next 8 bits are bit [10:3] of the bunch counter for the frame
    const CHIP_HEADER_RANGE: RangeInclusive<u8> = 0xA0..=0xAF;
    const CHIP_EMPTY_FRAME: u8 = 0xE0; // 1110_<chip_id[3:0]> next 8 bits are bit [10:3] of the bunch counter for the frame
    const CHIP_EMPTY_FRAME_RANGE: RangeInclusive<u8> = 0xE0..=0xEF;
    const CHIP_TRAILER: u8 = 0xB0; // 1011_<readout_flags[3:0]>
    const CHIP_TRAILER_RANGE: RangeInclusive<u8> = 0xB0..=0xBF;
    const REGION_HEADER: u8 = 0xC0; // 110<region_id[4:0]>
    const REGION_HEADER_RANGE: RangeInclusive<u8> = 0xC0..=0xDF;
    const DATA_SHORT: u8 = 0b0100_0000; // 01<encoder_id[3:0]> next 10 bits are <addr[9:0]>
    const DATA_SHORT_RANGE: RangeInclusive<u8> = 0x40..=0x7F;
    const DATA_LONG: u8 = 0b0000_0000; // 00<encoder_id[3:0]> next 18 bits are <addr[9:0]>_0_<hit_map[6:0]>
    const DATA_LONG_RANGE: RangeInclusive<u8> = 0x00..=0x3F;
    const BUSY_ON: u8 = 0xF0;
    const BUSY_OFF: u8 = 0xF1;

    #[inline]
    pub fn from_byte(b: u8) -> Result<AlpideWord, ()> {
        match b {
            c if c & 0xC0 == Self::DATA_SHORT => Ok(AlpideWord::DataShort),
            c if c & 0xC0 == Self::DATA_LONG => Ok(AlpideWord::DataLong),
            c if c & 0xE0 == Self::REGION_HEADER => Ok(AlpideWord::RegionHeader),
            c if c & 0xF0 == Self::CHIP_EMPTY_FRAME => Ok(AlpideWord::ChipEmptyFrame),
            c if Self::CHIP_HEADER_RANGE.contains(&c) => Ok(AlpideWord::ChipHeader),
            c if c & 0xF0 == Self::CHIP_TRAILER => Ok(AlpideWord::ChipTrailer),
            _ => Self::match_exact(b),
        }
    }

    #[inline]
    fn match_exact(b: u8) -> Result<AlpideWord, ()> {
        match b {
            Self::BUSY_ON => Ok(AlpideWord::BusyOn),
            Self::BUSY_OFF => Ok(AlpideWord::BusyOff),
            _ => AlpideProtocolExtension::from_byte(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn alpide_word_from_byte_variants() {
        assert_eq!(AlpideWord::from_byte(0xA0).unwrap(), AlpideWord::ChipHeader);
        assert_eq!(
            AlpideWord::from_byte(0xE0).unwrap(),
            AlpideWord::ChipEmptyFrame
        );
        assert_eq!(
            AlpideWord::from_byte(0xB0).unwrap(),
            AlpideWord::ChipTrailer
        );
        assert_eq!(
            AlpideWord::from_byte(0xC0).unwrap(),
            AlpideWord::RegionHeader
        );
        assert_eq!(
            AlpideWord::from_byte(0b0100_0000).unwrap(),
            AlpideWord::DataShort
        );
        assert_eq!(
            AlpideWord::from_byte(0b0000_0000).unwrap(),
            AlpideWord::DataLong
        );
        assert_eq!(AlpideWord::from_byte(0xF0).unwrap(), AlpideWord::BusyOn);
        assert_eq!(AlpideWord::from_byte(0xF1).unwrap(), AlpideWord::BusyOff);
    }

    #[test]
    fn alpide_word_from_byte_ape_variants() {
        // No test for padding as it is state dependent
        assert_eq!(
            AlpideWord::from_byte(0xF2).unwrap(),
            AlpideWord::Ape(AlpideProtocolExtension::StripStart)
        );
        assert_eq!(
            AlpideWord::from_byte(0xF4).unwrap(),
            AlpideWord::Ape(AlpideProtocolExtension::DetectorTimeout)
        );
        assert_eq!(
            AlpideWord::from_byte(0xF5).unwrap(),
            AlpideWord::Ape(AlpideProtocolExtension::OutOfTable)
        );
        assert_eq!(
            AlpideWord::from_byte(0xF6).unwrap(),
            AlpideWord::Ape(AlpideProtocolExtension::ProtocolError)
        );
        assert_eq!(
            AlpideWord::from_byte(0xF7).unwrap(),
            AlpideWord::Ape(AlpideProtocolExtension::LaneFifoOverflowError)
        );
        assert_eq!(
            AlpideWord::from_byte(0xF8).unwrap(),
            AlpideWord::Ape(AlpideProtocolExtension::FsmError)
        );
        assert_eq!(
            AlpideWord::from_byte(0xF9).unwrap(),
            AlpideWord::Ape(AlpideProtocolExtension::PendingDetectorEventLimit)
        );
        assert_eq!(
            AlpideWord::from_byte(0xFA).unwrap(),
            AlpideWord::Ape(AlpideProtocolExtension::PendingLaneEventLimit)
        );
        assert_eq!(
            AlpideWord::from_byte(0xFB).unwrap(),
            AlpideWord::Ape(AlpideProtocolExtension::O2nError)
        );
        assert_eq!(
            AlpideWord::from_byte(0xFC).unwrap(),
            AlpideWord::Ape(AlpideProtocolExtension::RateMissingTriggerError)
        );
        assert_eq!(
            AlpideWord::from_byte(0xFD).unwrap(),
            AlpideWord::Ape(AlpideProtocolExtension::PeDataMissing)
        );
        assert_eq!(
            AlpideWord::from_byte(0xFE).unwrap(),
            AlpideWord::Ape(AlpideProtocolExtension::OotDataMissing)
        );
    }

    #[test]
    fn alpide_word_from_byte_variants_ranges() {
        for b in 0xA0..=0xAF {
            assert_eq!(AlpideWord::from_byte(b).unwrap(), AlpideWord::ChipHeader);
        }
        for b in 0xE0..=0xEF {
            assert_eq!(
                AlpideWord::from_byte(b).unwrap(),
                AlpideWord::ChipEmptyFrame
            );
        }
        for b in 0xB0..=0xBF {
            assert_eq!(AlpideWord::from_byte(b).unwrap(), AlpideWord::ChipTrailer);
        }
        for b in 0xC0..=0xDF {
            assert_eq!(AlpideWord::from_byte(b).unwrap(), AlpideWord::RegionHeader);
        }
        for b in 0x40..=0x7F {
            assert_eq!(AlpideWord::from_byte(b).unwrap(), AlpideWord::DataShort);
        }
        for b in 0x00..=0x3F {
            assert_eq!(AlpideWord::from_byte(b).unwrap(), AlpideWord::DataLong);
        }
    }
}
