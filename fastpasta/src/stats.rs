//! All stat collecting functionality

use crate::util::*;

pub mod err_printer;
pub mod lib;
pub mod stats_collector;
pub(super) mod stats_report;
mod stats_validation;

#[derive(Debug, Clone, PartialEq)]
/// Possible stats that can be sent for stats collection.
pub enum StatType {
    /// Fatal error, stop processing.
    Fatal(Box<str>),
    /// Non-fatal error, reported but processing continues.
    Error(Box<str>),
    /// The first trigger type observed is the type of run the data comes from
    ///
    /// Contains the raw value and the string description summarizing the trigger type
    RunTriggerType((u32, Box<str>)),
    /// The trigger_type field observed in the `RDH`
    TriggerType(u32),
    /// The first system ID observed is the basis for the rest of processing
    SystemId(SystemId),
    /// Increment the total RDHs seen.
    RDHSeen(u32),
    /// Increment the total RDHs filtered.
    RDHFiltered(u32),
    /// Increment the total payload size.
    PayloadSize(u32),
    /// Add a link to the list of links observed.
    LinksObserved(u8),
    /// Record the RDH version detected.
    RdhVersion(u8),
    /// Record the data format detected.
    DataFormat(u8),
    /// Increment the total HBFs seen.
    HBFsSeen(u32),
    /// Record a layer/stave combination seen.
    LayerStaveSeen {
        /// The layer number.
        layer: u8,
        /// The stave number.
        stave: u8,
    },
    /// Record the generic FEE ID
    FeeId(u16),
    /// Stats from ALPIDE data analysis
    AlpideStats(AlpideStats),
}

impl fmt::Display for StatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatType::RunTriggerType((val, description)) => {
                write!(f, "Run trigger type: {val}: {description}")
            }
            StatType::RDHSeen(val) => write!(f, "{val} RDHs seen"),
            StatType::RDHFiltered(val) => write!(f, "{val} RDHs filtered"),
            StatType::PayloadSize(bytes) => write!(f, "Payload size: {bytes}"),
            StatType::LinksObserved(id) => write!(f, "Link observed: {id}"),
            StatType::RdhVersion(v) => write!(f, "RDH version: {v}"),
            StatType::DataFormat(format) => write!(f, "Data format: {format}"),
            StatType::HBFsSeen(val) => write!(f, "{val} HBFs seen"),
            StatType::LayerStaveSeen {
                layer: layer_id,
                stave: stave_id,
            } => write!(f, "Layer/stave seen: {layer_id}/{stave_id}"),
            StatType::FeeId(id) => write!(f, "FEE ID: {id}"),
            StatType::TriggerType(trig_val) => write!(f, "Trigger type: {trig_val:#X}"),
            StatType::AlpideStats(alpide_stats) => write!(f, "ALPIDE stats {alpide_stats:?}"),
            StatType::SystemId(s_id) => write!(f, "System ID: {s_id}"),
            StatType::Error(e) => write!(f, "Error: {e}"),
            StatType::Fatal(e) => write!(f, "Fatal error: {e}"),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// Enums to represent each subsystem in the ALICE DAQ from the System ID.
pub enum SystemId {
    // ignore missing docs
    TPC,
    TRD,
    TOF,
    HMP,
    PHS,
    CPV,
    MCH,
    ZDC,
    TRG,
    EMC,
    TST, // TEST
    ITS,
    FDD,
    FT0,
    FV0,
    MFT,
    MID,
    DCS,
    FOC, // Focal
    Unloaded,
}

impl SystemId {
    /// Create a System ID enum from a system ID value
    pub fn from_system_id(sys_id: u8) -> Result<Self, String> {
        match sys_id {
            3 => Ok(SystemId::TPC),
            4 => Ok(SystemId::TRD),
            5 => Ok(SystemId::TOF),
            6 => Ok(SystemId::HMP),
            7 => Ok(SystemId::PHS),
            8 => Ok(SystemId::CPV),
            10 => Ok(SystemId::MCH),
            15 => Ok(SystemId::ZDC),
            17 => Ok(SystemId::TRG),
            18 => Ok(SystemId::EMC),
            19 => Ok(SystemId::TST),
            32 => Ok(SystemId::ITS),
            33 => Ok(SystemId::FDD),
            34 => Ok(SystemId::FT0),
            35 => Ok(SystemId::FV0),
            36 => Ok(SystemId::MFT),
            37 => Ok(SystemId::MID),
            38 => Ok(SystemId::DCS),
            39 => Ok(SystemId::FOC),
            255 => Ok(SystemId::Unloaded),
            _ => Err(format!("Unknown system ID {sys_id}")),
        }
    }
}

impl fmt::Display for SystemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemId::TPC => write!(f, "TPC"),
            SystemId::TRD => write!(f, "TRD"),
            SystemId::TOF => write!(f, "TOF"),
            SystemId::HMP => write!(f, "HMP"),
            SystemId::PHS => write!(f, "PHS"),
            SystemId::CPV => write!(f, "CPV"),
            SystemId::MCH => write!(f, "MCH"),
            SystemId::ZDC => write!(f, "ZDC"),
            SystemId::TRG => write!(f, "TRG"),
            SystemId::EMC => write!(f, "EMC"),
            SystemId::TST => write!(f, "TST"),
            SystemId::ITS => write!(f, "ITS"),
            SystemId::FDD => write!(f, "FDD"),
            SystemId::FT0 => write!(f, "FT0"),
            SystemId::FV0 => write!(f, "FV0"),
            SystemId::MFT => write!(f, "MFT"),
            SystemId::MID => write!(f, "MID"),
            SystemId::DCS => write!(f, "DCS"),
            SystemId::FOC => write!(f, "FOC"),
            SystemId::Unloaded => write!(f, "Unloaded"),
        }
    }
}

/// Takes an [RDH] and determines the [SystemId] and collects system specific stats.
/// Uses the received [`Option<SystemId>`] to check if the system ID has already been determined,
/// otherwise it will determine the [SystemId] and send it via the channel [`flume::Sender<StatType>`].
///
/// ### Arguments
/// * `rdh` - The [RDH] to collect stats from.
/// * `system_id` - The [`Option<SystemId>`] to check if the system ID has already been determined.
/// * `stats_sender_channel` - The [`flume::Sender<StatType>`] to send the stats through.
/// ### Returns
/// * `Ok(())` - If the stats were collected successfully.
/// * `Err(())` - If its the first time the [SystemId] is determined and the [SystemId] is not recognized.
pub fn collect_system_specific_stats<T: RDH + 'static>(
    rdh: &T,
    system_id: &mut Option<SystemId>,
    stats_send_chan: &flume::Sender<StatType>,
) -> Result<(), String> {
    if system_id.is_none() {
        // First time seeing a system ID
        let observed_sys_id = match SystemId::from_system_id(rdh.rdh0().system_id) {
            Ok(id) => id,
            Err(e) => return Err(e),
        };
        *system_id = Some(observed_sys_id);
    }

    if let Some(system_id) = system_id {
        // Determine the system ID and collect system specific stats
        match system_id {
            // Collect stats for each system
            SystemId::ITS => {
                log::trace!("Collecting stats for ITS");
                collect_its_stats(rdh, stats_send_chan)
            }
            // Example for other systems (and to make clippy shut up about using if let instead of match, cause only 1 case is implemented)
            SystemId::FOC => {
                log::trace!("Collecting stats for Focal");
                // stat collection not implemented
            }
            _ => (), // Do nothing for other systems
        }
    } else {
        unreachable!("System ID should have been determined by now")
    }
    Ok(())
}

/// Collects stats specific to ITS from the given [RDH] and sends them via the channel [`flume::Sender<StatType>`].
fn collect_its_stats<T: RDH>(rdh: &T, stats_send_chan: &flume::Sender<StatType>) {
    let layer = words::its::layer_from_feeid(rdh.fee_id());
    let stave = words::its::stave_number_from_feeid(rdh.fee_id());
    stats_send_chan
        .send(StatType::LayerStaveSeen { layer, stave })
        .unwrap();
}
