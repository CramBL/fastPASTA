//! Contains the [init_stats_controller] function, which spawns a thread with the [StatsController] running, and returns the thread handle, the channel to send stats to, and the stop flag.
use super::stats_controller::StatsController;
use crate::{util::lib::Config, words};
use std::sync::atomic::AtomicBool;

/// Spawns a thread with the StatsController running, and returns the thread handle, the channel to send stats to, and the stop flag.
pub fn init_stats_controller<C: Config + 'static>(
    config: std::sync::Arc<C>,
) -> (
    std::thread::JoinHandle<()>,
    std::sync::mpsc::Sender<StatType>,
    std::sync::Arc<AtomicBool>,
) {
    let mut stats = StatsController::new(config);
    let send_stats_channel = stats.send_channel();
    let thread_stop_flag = stats.end_processing_flag();
    let stats_thread = std::thread::Builder::new()
        .name("stats_thread".to_string())
        .spawn(move || {
            stats.run();
        })
        .expect("Failed to spawn stats thread");
    (stats_thread, send_stats_channel, thread_stop_flag)
}

// Stat collection functions

/// Collects stats specific to ITS from the given [RDH][words::lib::RDH] and sends them to the [StatsController].
pub fn collect_its_stats<T: words::lib::RDH>(
    rdh: &T,
    stats_sender_channel: &std::sync::mpsc::Sender<StatType>,
) {
    let layer = words::its::layer_from_feeid(rdh.fee_id());
    let stave = words::its::stave_number_from_feeid(rdh.fee_id());
    stats_sender_channel
        .send(StatType::LayerStaveSeen { layer, stave })
        .unwrap();
    stats_sender_channel
        .send(StatType::DataFormat(rdh.data_format()))
        .unwrap();
}

#[allow(missing_docs)]
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

    /// Convert a System ID enum to a string
    pub fn to_string(&self) -> String {
        match *self {
            SystemId::TPC => format!("TPC"),
            SystemId::TRD => format!("TRD"),
            SystemId::TOF => format!("TOF"),
            SystemId::HMP => format!("HMP"),
            SystemId::PHS => format!("PHS"),
            SystemId::CPV => format!("CPV"),
            SystemId::MCH => format!("MCH"),
            SystemId::ZDC => format!("ZDC"),
            SystemId::TRG => format!("TRG"),
            SystemId::EMC => format!("EMC"),
            SystemId::TST => format!("TST"),
            SystemId::ITS => format!("ITS"),
            SystemId::FDD => format!("FDD"),
            SystemId::FT0 => format!("FT0"),
            SystemId::FV0 => format!("FV0"),
            SystemId::MFT => format!("MFT"),
            SystemId::MID => format!("MID"),
            SystemId::DCS => format!("DCS"),
            SystemId::FOC => format!("FOC"),
            SystemId::Unloaded => format!("Unloaded"),
        }
    }
}

/// Possible stats that can be sent to the StatsController.
pub enum StatType {
    /// Fatal error, stop processing.
    Fatal(String),
    /// Non-fatal error, reported but processing continues.
    Error(String),
    /// The first trigger type observed is the type of run the data comes from
    ///
    /// Contains the raw value and the string description summarizing the trigger type
    RunTriggerType((u32, String)),
    /// Increment the total RDHs seen.
    RDHsSeen(u8),
    /// Increment the total RDHs filtered.
    RDHsFiltered(u8),
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
}
