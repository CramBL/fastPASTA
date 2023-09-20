//! Contains the [StatsCollector] that collects stats from analysis.
pub(super) mod error_stats;
pub mod its_stats;
pub(super) mod rdh_stats;
pub(super) mod trigger_stats;

use crate::config::custom_checks::CustomChecksOpt;

use super::stats_validation::validate_custom_stats;
use super::{StatType, SystemId};
use error_stats::ErrorStats;
use its_stats::alpide_stats::AlpideStats;
use rdh_stats::RdhStats;
use serde::{Deserialize, Serialize};

/// Collects stats from analysis.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StatsCollector {
    is_finalized: bool,
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

    pub(super) fn validate_custom_stats(&mut self, custom_checks: &'static impl CustomChecksOpt) {
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
    pub fn finalize(&mut self) {
        if self.is_finalized {
            return;
        }
        self.error_stats.finalize_stats();
        self.rdh_stats.finalize();

        // If the data is from ITS, correlate the errors with the layer/stave
        if matches!(self.rdh_stats().system_id(), Some(SystemId::ITS)) {
            self.error_stats
                .check_errors_for_stave_id(self.rdh_stats.layer_staves_as_slice());
        }

        self.is_finalized = true;
    }

    pub(super) fn rdh_stats(&self) -> &RdhStats {
        &self.rdh_stats
    }

    pub(super) fn error_stats(&self) -> &ErrorStats {
        &self.error_stats
    }

    pub(super) fn total_errors(&self) -> u64 {
        self.error_stats.total_errors()
    }

    pub(super) fn is_fatal_err(&self) -> bool {
        self.error_stats.is_fatal_error()
    }

    pub(super) fn consume_reported_errors(&mut self) -> Vec<Box<str>> {
        self.error_stats.consume_reported_errors()
    }

    pub(super) fn take_fatal_err(&mut self) -> Box<str> {
        self.error_stats.take_fatal_err()
    }

    pub(super) fn unique_error_codes_as_slice(&mut self) -> &[u16] {
        self.error_stats.unique_error_codes_as_slice()
    }

    pub(super) fn take_alpide_stats(&mut self) -> Option<AlpideStats> {
        self.alpide_stats.take()
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
        stats_collector.finalize();

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
}
