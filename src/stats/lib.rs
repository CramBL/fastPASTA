//! Contains the [init_stats_controller] function, which spawns a thread with the [StatsController] running, and returns the thread handle, the channel to send stats to, and the stop flag.
use super::stats_controller::StatsController;
use crate::{util::lib::Config, words};
use std::{fmt::Display, sync::atomic::AtomicBool};

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

// Stat collection functionality

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
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

impl std::fmt::Display for SystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

#[derive(Debug, Clone, PartialEq)]
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
    /// The first system ID observed is the basis for the rest of processing
    SystemId(SystemId),
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

impl Display for StatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatType::Fatal(e) => write!(f, "Fatal error: {e}"),
            StatType::Error(e) => write!(f, "Error: {e}"),
            StatType::RunTriggerType((val, description)) => {
                write!(f, "Run trigger type: {val}: {description}")
            }
            StatType::SystemId(s_id) => write!(f, "System ID: {s_id}"),
            StatType::RDHsSeen(cnt) => write!(f, "RDHs seen: {cnt}"),
            StatType::RDHsFiltered(cnt) => write!(f, "RDHs filtered: {cnt}"),
            StatType::PayloadSize(bytes) => write!(f, "Payload size: {bytes}"),
            StatType::LinksObserved(id) => write!(f, "Link observed: {id}"),
            StatType::RdhVersion(v) => write!(f, "RDH version: {v}"),
            StatType::DataFormat(format) => write!(f, "Data format: {format}"),
            StatType::HBFsSeen(cnt) => write!(f, "HBFs seen: {cnt}"),
            StatType::LayerStaveSeen {
                layer: layer_id,
                stave: stave_id,
            } => write!(f, "Layer/stave seen: {layer_id}/{stave_id}"),
        }
    }
}

/// Takes an [RDH](words::lib::RDH) and determines the [SystemId] and collects system specific stats.
/// Uses the received [`Option<SystemId>`] to check if the system ID has already been determined,
/// otherwise it will determine the [SystemId] and send it to the [StatsController](StatsController) via the channel [`std::sync::mpsc::Sender<StatType>`].
///
/// # Arguments
/// * `rdh` - The [RDH](words::lib::RDH) to collect stats from.
/// * `system_id` - The [`Option<SystemId>`] to check if the system ID has already been determined.
/// * `stats_sender_channel` - The [`std::sync::mpsc::Sender<StatType>`] to send the stats to the [StatsController](StatsController).
/// # Returns
/// * `Ok(())` - If the stats were collected successfully.
/// * `Err(())` - If its the first time the [SystemId] is determined and the [SystemId] is not recognized.
pub fn collect_system_specific_stats<T: words::lib::RDH + 'static>(
    rdh: &T,
    system_id: &mut Option<SystemId>,
    stats_sender_channel: &std::sync::mpsc::Sender<StatType>,
) -> Result<(), String> {
    if let Some(system_id) = system_id {
        // Determine the system ID and collect system specific stats
        match system_id {
            // Collect stats for each system
            SystemId::ITS => {
                log::trace!("Collecting stats for ITS");
                collect_its_stats(rdh, stats_sender_channel)
            }
            // Example for other systems (and to make clippy shut up about using if let instead of match, cause only 1 case is implemented)
            SystemId::FOC => {
                log::trace!("Collecting stats for Focal");
                // stat collection not implemented
            }
            _ => (), // Not implemented
        }
    } else {
        // First time seeing a system ID
        let observed_sys_id = match SystemId::from_system_id(rdh.rdh0().system_id) {
            Ok(id) => id,
            Err(e) => return Err(e),
        };
        log::info!("{observed_sys_id} detected");
        stats_sender_channel
            .send(StatType::SystemId(observed_sys_id))
            .unwrap();
        *system_id = Some(observed_sys_id);
    }
    Ok(())
}

/// Collects stats specific to ITS from the given [RDH][words::lib::RDH] and sends them to the [StatsController].
fn collect_its_stats<T: words::lib::RDH>(
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

#[cfg(test)]
mod tests {
    use crate::words::lib::RDH;

    use super::*;

    #[test]
    fn test_collect_its_stats() {
        let (stats_sender, stats_receiver) = std::sync::mpsc::channel::<StatType>();
        let rdh = crate::words::rdh_cru::test_data::CORRECT_RDH_CRU_V7;

        let expect_layer = crate::words::its::layer_from_feeid(rdh.fee_id());
        let expect_stave = crate::words::its::stave_number_from_feeid(rdh.fee_id());

        collect_its_stats(&rdh, &stats_sender);

        let stats = stats_receiver.recv().unwrap();

        match stats {
            StatType::LayerStaveSeen { layer, stave } => {
                assert_eq!(layer, expect_layer);
                assert_eq!(stave, expect_stave);
            }
            _ => panic!("Wrong stat type received"),
        }
    }

    #[test]
    fn test_collect_system_specific_stats() {
        let (stats_sender, stats_receiver) = std::sync::mpsc::channel::<StatType>();
        let mut system_id = None;

        let rdh = crate::words::rdh_cru::test_data::CORRECT_RDH_CRU_V7;

        collect_system_specific_stats(&rdh, &mut system_id, &stats_sender).unwrap();

        let stats = stats_receiver.recv().unwrap();

        match stats {
            StatType::SystemId(id) => assert_eq!(id, SystemId::ITS),
            _ => panic!("Wrong stat type received"),
        }
    }
}
