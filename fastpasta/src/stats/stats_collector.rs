#![allow(dead_code)]
//! Contains the [StatsCollector] that collects stats from analysis.
pub mod error_stats;
pub mod its_stats;
pub mod rdh_stats;
pub mod trigger_stats;

use crate::config::custom_checks::CustomChecksOpt;
use crate::config::inputoutput::{DataOutputFormat, DataOutputMode};

use super::stats_validation::validate_custom_stats;
use super::{StatType, SystemId};
use error_stats::ErrorStats;
use its_stats::alpide_stats::AlpideStats;
use rdh_stats::RdhStats;
use serde::{Deserialize, Serialize};

/// Collects stats from analysis.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StatsCollector {
    /// If the stats collection is finalized. If finalized, no more stats can be collected. If it is not finalized, it is not valid to read the stats.
    pub is_finalized: bool,
    rdh_stats: RdhStats,
    error_stats: ErrorStats,
    alpide_stats: Option<AlpideStats>,
}

impl StatsCollector {
    /// Create a new StatsCollector that includes ALPIDE stats.
    /// Only used if checks on ALPIDE data is enabled.
    pub fn with_alpide_stats() -> Self {
        Self {
            alpide_stats: Some(AlpideStats::default()),
            ..Default::default()
        }
    }

    /// Record a stat.
    pub fn collect(&mut self, stat: StatType) {
        match stat {
            StatType::Fatal(m) => self.error_stats.add_fatal_err(m),
            StatType::Error(m) => self.error_stats.add_err(m),
            StatType::RunTriggerType((raw_trigger_type, trigger_type_str)) => self
                .rdh_stats
                .record_run_trigger_type((raw_trigger_type, trigger_type_str)),
            StatType::TriggerType(t) => self.rdh_stats.record_trigger_type(t),
            StatType::SystemId(id) => self.rdh_stats.record_system_id(id),
            StatType::RDHSeen(e) => self.rdh_stats.add_rdhs_seen(e),
            StatType::RDHFiltered(e) => self.rdh_stats.add_rdhs_filtered(e),
            StatType::PayloadSize(sz) => self.rdh_stats.add_payload_size(sz as u64),
            StatType::LinksObserved(id) => self.rdh_stats.record_link(id),
            StatType::RdhVersion(v) => self.rdh_stats.record_rdh_version(v),
            StatType::DataFormat(v) => self.rdh_stats.record_data_format(v),
            StatType::HBFSeen => self.rdh_stats.incr_hbf_seen(),
            StatType::LayerStaveSeen { layer, stave } => {
                self.rdh_stats.record_layer_stave_seen((layer, stave))
            }
            StatType::FeeId(id) => self.rdh_stats.record_fee_observed(id),
            StatType::AlpideStats(s) => self.alpide_stats.as_mut().unwrap().sum(s),
        }
    }

    pub(crate) fn validate_custom_stats(&mut self, custom_checks: &'static impl CustomChecksOpt) {
        if let Err(e) = validate_custom_stats(custom_checks, &self.rdh_stats) {
            e.into_iter().for_each(|error_msg| {
                self.error_stats.add_custom_check_error(error_msg);
            });
        }
    }

    /// Finalize stats collection. Meaning no more stats can be collected.
    ///
    /// Does post-processing on the stats collected which assumes that no more stats are collected.
    /// Does nothing if already finalized.
    pub fn finalize(&mut self, mute_errors: bool) {
        if self.is_finalized {
            return;
        }
        self.rdh_stats.finalize();

        if matches!(self.rdh_stats().system_id(), Some(SystemId::ITS)) {
            self.error_stats
                .finalize_stats(mute_errors, Some(self.rdh_stats.layer_staves_as_slice()));
        } else {
            self.error_stats.finalize_stats(mute_errors, None);
        }

        self.is_finalized = true;
    }

    /// Display the errors reported, optionally limiting the number of errors displayed.
    pub fn display_errors(&self, display_max: Option<usize>, mute_errors: bool) {
        if mute_errors {
            return;
        }
        if let Some(max) = display_max {
            let mut cnt: usize = self.err_count() as usize;
            self.reported_errors_as_slice()
                .iter()
                .take(max)
                .for_each(|err| {
                    super::lib::display_error(err);
                    cnt -= 1;
                });

            if cnt > 0 {
                self.custom_check_errors_as_slice()
                    .iter()
                    .take(cnt)
                    .for_each(|err| {
                        super::lib::display_error(err);
                    });
            }
        } else {
            // Display all
            self.reported_errors_as_slice()
                .iter()
                .for_each(|err| super::lib::display_error(err));
            self.custom_check_errors_as_slice()
                .iter()
                .for_each(|err| super::lib::display_error(err));
        }
    }

    /// Returns a reference to the [RdhStats].
    pub fn rdh_stats(&self) -> &RdhStats {
        &self.rdh_stats
    }

    /// Returns the number of RDHs seen.
    pub fn rdhs_seen(&self) -> u64 {
        self.rdh_stats.rdhs_seen()
    }

    /// Returns the processed payload size in bytes
    pub fn payload_size(&self) -> u64 {
        self.rdh_stats.payload_size()
    }

    /// Returns the number of HBFs in the processed data.
    pub fn hbfs_seen(&self) -> u32 {
        self.rdh_stats.hbfs_seen()
    }

    /// Returns if any RDHs were seen in the processed data.
    pub fn any_rdhs_seen(&self) -> bool {
        self.rdh_stats.rdhs_seen() > 0
    }

    /// Returns the System ID of the processed data if it was observed/determined.
    pub fn system_id(&self) -> Option<SystemId> {
        self.rdh_stats.system_id()
    }

    /// Returns the layers/staves seen in the processed data as a borrowed slice.
    pub fn layer_staves_as_slice(&self) -> &[(u8, u8)] {
        self.rdh_stats.layer_staves_as_slice()
    }

    /// Returns a reference to the [ErrorStats].
    pub fn error_stats(&self) -> &ErrorStats {
        &self.error_stats
    }

    /// Returns the number of errors reported.
    pub fn err_count(&self) -> u64 {
        self.error_stats.err_count()
    }

    /// Return if any errors were reported.
    pub fn any_errors(&self) -> bool {
        self.error_stats.err_count() > 0
    }

    /// Returns if any fatal errors were reported.
    pub fn any_fatal_err(&self) -> bool {
        self.error_stats.any_fatal_err()
    }

    /// Returns a borrowed slice of the reported error messages as read-only strings.
    pub fn reported_errors_as_slice(&self) -> &[Box<str>] {
        self.error_stats.reported_errors_as_slice()
    }

    /// Returns a borrowed slice of the custom checks error messages as read-only strings.
    pub fn custom_check_errors_as_slice(&self) -> &[Box<str>] {
        self.error_stats.custom_check_errors_as_slice()
    }

    /// Returns a reference to the fatal error message.
    pub fn fatal_err(&self) -> &str {
        self.error_stats.fatal_err()
    }

    /// Returns a slice of the unique error codes of reported errors.
    pub fn unique_error_codes_as_slice(&mut self) -> &[u16] {
        self.error_stats.unique_error_codes_as_slice()
    }

    /// Returns a slice of the staves in which errors were reported.
    pub fn staves_with_errors_as_slice(&self) -> Option<&[(u8, u8)]> {
        self.error_stats.staves_with_errors_as_slice()
    }

    /// Returns a reference to the [AlpideStats] instance.
    pub fn alpide_stats(&self) -> Option<&AlpideStats> {
        self.alpide_stats.as_ref()
    }

    pub(crate) fn write_stats(&self, mode: &DataOutputMode, format: DataOutputFormat) {
        if *mode == DataOutputMode::None {
            return;
        }
        match format {
            DataOutputFormat::JSON => write_stats_str(
                mode,
                &serde_json::to_string_pretty(&self).expect("Failed to serialize stats to JSON"),
            ),
            DataOutputFormat::TOML => write_stats_str(
                mode,
                &toml::to_string_pretty(&self).expect("Failed to serialize stats to TOML"),
            ),
        }
    }

    /// Validate that the other stats (from user input) matches the collected stats.
    pub fn validate_other_stats(
        &self,
        other: &Self,
        mute_errors: bool,
    ) -> Result<(), std::io::Error> {
        let mut errs = Vec::new();

        if let Err(mut err_msgs) = self.rdh_stats.validate_other(other.rdh_stats()) {
            errs.append(&mut err_msgs);
        }

        if let Err(mut err_msgs) = self.error_stats.validate_other(other.error_stats()) {
            errs.append(&mut err_msgs);
        }

        if let Some(alpide_stats) = self.alpide_stats() {
            if let Some(other_alpide_stats) = other.alpide_stats() {
                if let Err(mut err_msgs) = alpide_stats.validate_other(other_alpide_stats) {
                    errs.append(&mut err_msgs);
                }
            } else {
                errs.push(
                    "ALPIDE stats was collected but the input stats does not contain ALPIDE stats"
                        .into(),
                );
            }
        } else if other.alpide_stats.is_some() {
            log::warn!("Input stats contains ALPIDE stats but the chosen analysis did not collect ALPIDE stats (did you mean to use `check all its-stave`?)");
        }

        if errs.is_empty() {
            Ok(())
        } else {
            if !mute_errors {
                errs.iter().for_each(|err| {
                    super::lib::display_error(err);
                });
            }
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Stats validation failed",
            ))
        }
    }
}

fn write_stats_str(mode: &DataOutputMode, stats_str: &str) {
    match mode {
        DataOutputMode::File(path) => {
            std::fs::write(path, stats_str).expect("Failed writing stats output file")
        }
        DataOutputMode::Stdout => println!("{stats_str}"),
        DataOutputMode::None => (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_serde() {
        // Test serialization and deserialization of StatsCollector to JSON and TOML.
        let mut stats_collector = StatsCollector::with_alpide_stats();
        stats_collector.collect(StatType::Fatal("fatal error".into()));
        stats_collector.collect(StatType::Error("error".into()));
        stats_collector.collect(StatType::RunTriggerType((0, "trigger type".into())));
        stats_collector.collect(StatType::TriggerType(0xE021));
        stats_collector.collect(StatType::SystemId(SystemId::ZDC));
        stats_collector.collect(StatType::RDHSeen(1));
        stats_collector.collect(StatType::RDHFiltered(2));
        stats_collector.collect(StatType::PayloadSize(3));
        stats_collector.collect(StatType::LinksObserved(4));
        stats_collector.collect(StatType::RdhVersion(5));
        stats_collector.collect(StatType::DataFormat(2));
        stats_collector.collect(StatType::HBFSeen);
        stats_collector.collect(StatType::LayerStaveSeen { layer: 6, stave: 7 });
        stats_collector.collect(StatType::FeeId(8));
        stats_collector.collect(StatType::AlpideStats(AlpideStats::default()));
        stats_collector.finalize(false);

        let json = serde_json::to_string(&stats_collector).unwrap();
        let from_json = serde_json::from_str::<StatsCollector>(&json).unwrap();
        println!(
            "{}",
            serde_json::to_string_pretty(&stats_collector).unwrap()
        );
        assert_eq!(stats_collector, from_json);

        let toml = toml::to_string(&stats_collector).unwrap();
        let from_toml = toml::from_str::<StatsCollector>(&toml).unwrap();
        println!("{toml}");
        assert_eq!(stats_collector, from_toml);
    }

    #[test]
    fn test_validate_other_stats_default_succeeds() {
        let stats_collector = StatsCollector::default();
        let other_stats_collector = StatsCollector::default();

        assert!(stats_collector
            .validate_other_stats(&other_stats_collector, false)
            .is_ok());
    }

    #[test]
    fn test_validate_other_stats_with_alpide_stats_succeeds() {
        let stats_collector = StatsCollector::with_alpide_stats();
        let other_stats_collector = StatsCollector::with_alpide_stats();

        assert!(stats_collector
            .validate_other_stats(&other_stats_collector, false)
            .is_ok());
    }

    #[test]
    fn test_validate_other_stats_with_alpide_stats_and_default_fails() {
        let stats_collector = StatsCollector::with_alpide_stats();
        let other_stats_collector = StatsCollector::default();

        if let Err(msgs) = stats_collector.validate_other_stats(&other_stats_collector, false) {
            println!("{:?}", msgs);
        } else {
            panic!("Should have failed");
        }

        // Vice versa is OK (if ALPIDE stats are provided by user, it's OK if the analysis didn't collect ALPIDE stats)
        assert!(other_stats_collector
            .validate_other_stats(&stats_collector, false)
            .is_ok());
        // The user should be warned about this though
    }

    #[test]
    fn test_validate_mismatch_errors() {
        let stats_collector = StatsCollector::default();
        let mut other_stats_collector = StatsCollector::default();

        other_stats_collector.collect(StatType::Fatal("fatal error".into()));
        other_stats_collector.collect(StatType::Error("error".into()));
        other_stats_collector.collect(StatType::RunTriggerType((0, "trigger type".into())));
        other_stats_collector.collect(StatType::TriggerType(0xE021));
        other_stats_collector.collect(StatType::SystemId(SystemId::ZDC));

        if let Err(err) = stats_collector.validate_other_stats(&other_stats_collector, false) {
            println!("{:?}", err);
        } else {
            panic!("Should have failed");
        }
    }

    #[test]
    fn test_validate_match_various_vals() {
        let mut stats_collector = StatsCollector::with_alpide_stats();
        let mut other_stats_collector = StatsCollector::with_alpide_stats();

        stats_collector.collect(StatType::Error("0xe0 : [E10] big error".into()));
        other_stats_collector.collect(StatType::Error("0xe0 : [E10] big error".into()));

        stats_collector.collect(StatType::FeeId(1));
        other_stats_collector.collect(StatType::FeeId(1));

        stats_collector.collect(StatType::RDHSeen(11));
        other_stats_collector.collect(StatType::RDHSeen(11));

        stats_collector.collect(StatType::RDHFiltered(10));
        other_stats_collector.collect(StatType::RDHFiltered(10));

        stats_collector.collect(StatType::PayloadSize(100));
        other_stats_collector.collect(StatType::PayloadSize(100));

        let mut alpide_stats = AlpideStats::default();
        alpide_stats.log_readout_flags(0b0000_0001);
        alpide_stats.log_readout_flags(0b0001_1001);

        stats_collector.collect(StatType::AlpideStats(alpide_stats));
        other_stats_collector.collect(StatType::AlpideStats(alpide_stats));

        assert!(stats_collector
            .validate_other_stats(&other_stats_collector, false)
            .is_ok());
    }
}
