//! Contains the [init_stats_controller] function, which spawns a thread with the [StatsController] running, and returns the thread handle, the channel to send stats to, and the stop flag.
use super::stats_controller::StatsController;
use crate::{util::lib::Config, words};
use std::{fmt::Display, sync::atomic::AtomicBool};

/// Spawns a thread with the StatsController running, and returns the thread handle, the channel to send stats to, and the stop flag.
pub fn init_stats_controller<C: Config + 'static>(
    config: &'static C,
) -> (
    std::thread::JoinHandle<()>,
    flume::Sender<StatType>,
    std::sync::Arc<AtomicBool>,
    std::sync::Arc<AtomicBool>,
) {
    log::trace!("Initializing stats controller");
    let mut stats = StatsController::new(config);
    let send_stats_channel = stats.send_channel();
    let thread_stop_flag = stats.end_processing_flag();
    let any_errors_flag = stats.any_errors_flag();

    let stats_thread = std::thread::Builder::new()
        .name("stats_thread".to_string())
        .spawn(move || {
            stats.run();
        })
        .expect("Failed to spawn stats thread");
    (
        stats_thread,
        send_stats_channel,
        thread_stop_flag,
        any_errors_flag,
    )
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
    /// Record the generic FEE ID
    FeeId(u16),
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
            StatType::FeeId(id) => write!(f, "FEE ID: {id}"),
        }
    }
}

/// Takes an [RDH](words::lib::RDH) and determines the [SystemId] and collects system specific stats.
/// Uses the received [`Option<SystemId>`] to check if the system ID has already been determined,
/// otherwise it will determine the [SystemId] and send it to the [StatsController](StatsController) via the channel [`flume::Sender<StatType>`].
///
/// # Arguments
/// * `rdh` - The [RDH](words::lib::RDH) to collect stats from.
/// * `system_id` - The [`Option<SystemId>`] to check if the system ID has already been determined.
/// * `stats_sender_channel` - The [`flume::Sender<StatType>`] to send the stats to the [StatsController](StatsController).
/// # Returns
/// * `Ok(())` - If the stats were collected successfully.
/// * `Err(())` - If its the first time the [SystemId] is determined and the [SystemId] is not recognized.
pub fn collect_system_specific_stats<T: words::lib::RDH + 'static>(
    rdh: &T,
    system_id: &mut Option<SystemId>,
    stats_sender_channel: &flume::Sender<StatType>,
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
                collect_its_stats(rdh, stats_sender_channel)
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

/// Collects stats specific to ITS from the given [RDH][words::lib::RDH] and sends them to the [StatsController].
fn collect_its_stats<T: words::lib::RDH>(rdh: &T, stats_sender_channel: &flume::Sender<StatType>) {
    let layer = words::its::layer_from_feeid(rdh.fee_id());
    let stave = words::its::stave_number_from_feeid(rdh.fee_id());
    stats_sender_channel
        .send(StatType::LayerStaveSeen { layer, stave })
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::words::lib::RDH_CRU;
    use std::sync::OnceLock;

    #[test]
    fn test_collect_its_stats() {
        let (stats_sender, stats_receiver) = flume::unbounded::<StatType>();
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
        let (stats_sender, stats_receiver) = flume::unbounded::<StatType>();
        let mut system_id = None;

        let rdh = crate::words::rdh_cru::test_data::CORRECT_RDH_CRU_V7;
        let expect_layer = crate::words::its::layer_from_feeid(rdh.fee_id());
        let expect_stave = crate::words::its::stave_number_from_feeid(rdh.fee_id());

        collect_system_specific_stats(&rdh, &mut system_id, &stats_sender).unwrap();

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
    fn test_system_id_from_system_id() {
        let system_id = SystemId::from_system_id(32).unwrap();
        assert_eq!(system_id, SystemId::ITS);
        let as_string = format!("{system_id}");
        assert_eq!(as_string, "ITS");
    }

    use crate::util::lib::test_util::MockConfig;
    static CONFIG_TEST_INIT_STATS_CONTROLLER: OnceLock<MockConfig> = OnceLock::new();
    #[test]
    fn test_init_stats_controller() {
        let mock_config = MockConfig::default();
        CONFIG_TEST_INIT_STATS_CONTROLLER.set(mock_config).unwrap();

        let (handle, send_ch, stop_flag, _errors_flag) =
            init_stats_controller(CONFIG_TEST_INIT_STATS_CONTROLLER.get().unwrap());

        // Stop flag should be false
        assert!(!stop_flag.load(std::sync::atomic::Ordering::SeqCst));

        // Send RDH version seen
        send_ch.send(StatType::RdhVersion(7)).unwrap();

        // Send Data format seen
        send_ch.send(StatType::DataFormat(99)).unwrap();

        // Send Run Trigger Type
        send_ch
            .send(StatType::RunTriggerType((0xBEEF, "BEEF".to_owned())))
            .unwrap();

        // Send rdh seen stat
        send_ch.send(StatType::RDHsSeen(1)).unwrap();

        // Send a fatal error that should cause the stop flag to be set
        send_ch
            .send(StatType::Fatal("Test fatal error".to_string()))
            .unwrap();

        // Stop the controller by dropping the sender channel
        drop(send_ch);

        // Wait for the controller to stop
        handle.join().unwrap();

        // Stop flag should be true
        assert!(stop_flag.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_system_id_enums_all() {
        let valid_system_ids: [u8; 20] = [
            3, 4, 5, 6, 7, 8, 10, 15, 17, 18, 19, 32, 33, 34, 35, 36, 37, 38, 39, 255,
        ];
        for id in 0..=255 {
            let system_id = SystemId::from_system_id(id);
            if valid_system_ids.contains(&id) {
                assert!(system_id.is_ok());
                let to_str = system_id.unwrap().to_string();
                assert!(!to_str.is_empty());
            } else {
                assert!(system_id.is_err());
                let to_str = system_id.unwrap_err().to_string();
                assert_eq!(to_str, format!("Unknown system ID {id}"));
            }
        }
    }

    #[test]
    fn test_all_stattype_enums() {
        let fatal = StatType::Fatal("Test fatal error".to_string());
        let error = StatType::Error("Test error".to_string());
        let run_trig_type = StatType::RunTriggerType((1, "Test run trigger type".to_string()));
        let sys_id = StatType::SystemId(SystemId::ITS);
        let rdh_seen = StatType::RDHsSeen(1);
        let rdh_filtered = StatType::RDHsFiltered(1);
        let layer_stave_seen = StatType::LayerStaveSeen { layer: 1, stave: 1 };
        let mut stat_type_vec = vec![
            fatal,
            error,
            run_trig_type,
            sys_id,
            rdh_seen,
            rdh_filtered,
            layer_stave_seen,
        ];

        stat_type_vec.push(StatType::PayloadSize(1));
        stat_type_vec.push(StatType::LinksObserved(0));
        stat_type_vec.push(StatType::RdhVersion(1));
        stat_type_vec.push(StatType::DataFormat(1));
        stat_type_vec.push(StatType::HBFsSeen(2));

        for stat_type in stat_type_vec {
            // Test to_string() method
            let to_str = stat_type.to_string();
            assert!(!to_str.is_empty());
        }
    }
}
