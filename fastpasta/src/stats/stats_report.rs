pub mod report;
pub(super) mod stat_format_utils;
pub(super) mod stat_summerize_utils;
mod table_formatter_utils;

use self::{
    stat_format_utils::{format_error_codes, format_fee_ids, format_links_observed},
    stat_summerize_utils::{
        summerize_data_size, summerize_filtered_fee_ids, summerize_filtered_its_layer_staves,
        summerize_filtered_links, summerize_layers_staves_seen,
    },
};
use crate::util::*;

/// Helper function that makes the report
pub fn make_report(
    processing_time: Duration,
    stats: &mut StatsCollector,
    filter_target: Option<FilterTarget>,
) -> Report {
    debug_assert!(stats.is_finalized);

    let mut report = Report::new(processing_time);

    if stats.any_fatal_err() {
        report.add_fatal_error(stats.fatal_err().to_owned());
    }

    // Add global stats
    add_global_stats_to_report(&mut report, stats);

    if filter_target.is_some() {
        let filtered_stats: Vec<StatSummary> = add_filtered_stats(stats, filter_target);
        report.add_filter_stats(tabled::Table::new(filtered_stats));
    } else {
        // Check if the observed system ID is ITS
        if matches!(stats.system_id(), Some(SystemId::ITS)) {
            // If no filtering, the layers and staves seen is from the total RDHs
            report.add_stat(summerize_layers_staves_seen(
                stats.layer_staves_as_slice(),
                stats.staves_with_errors_as_slice(),
            ));
        }
        // If no filtering, the HBFs seen is from the total RDHs
        report.add_stat(StatSummary::new(
            "Total HBFs".to_string(),
            stats.hbfs_seen().to_string(),
            None,
        ));

        // If no filtering, the payload size seen is from the total RDHs
        report.add_stat(summerize_data_size(stats.rdhs_seen(), stats.payload_size()));
    }

    // Add ALPIDE stats (if they are collected)
    if let Some(alpide_stats) = stats.alpide_stats() {
        add_alpide_stats_to_report(&mut report, alpide_stats);
    }

    // Add detected attributes
    add_detected_attributes_to_report(&mut report, stats.rdh_stats());

    report
}

/// Helper function that adds the global stats to the report
fn add_global_stats_to_report(report: &mut Report, stats: &mut StatsCollector) {
    if stats.err_count() == 0 {
        report.add_stat(StatSummary::new(
            "Total Errors".green().to_string(),
            stats.err_count().green().to_string(),
            None,
        ));
    } else {
        report.add_stat(StatSummary::new(
            "Total Errors".red().to_string(),
            stats.err_count().red().to_string(),
            Some(format_error_codes(stats.unique_error_codes_as_slice())),
        ));
    }

    let (trigger_type_raw, trigger_type_str) = stats.rdh_stats().run_trigger_type();
    report.add_stat(StatSummary {
        statistic: "Run Trigger Type".to_string(),
        value: format!("{trigger_type_raw:#02X}"),
        notes: trigger_type_str.into_string(),
    });
    report.add_stat(StatSummary::new(
        "Total RDHs".to_string(),
        stats.rdh_stats().rdhs_seen().to_string(),
        None,
    ));
    report.add_stat(StatSummary::new(
        "Links observed".to_string(),
        format_links_observed(stats.rdh_stats().links_as_slice()),
        None,
    ));
    report.add_stat(StatSummary::new(
        "FEE IDs seen".to_string(),
        format_fee_ids(stats.rdh_stats().fee_ids_as_slice()),
        None,
    ));
}

/// Helper function that builds a vector of the stats associated with the filtered data
fn add_filtered_stats(
    stats: &StatsCollector,
    filter_target: Option<FilterTarget>,
) -> Vec<StatSummary> {
    let mut filtered_stats: Vec<StatSummary> = Vec::new();
    filtered_stats.push(StatSummary::new(
        "RDHs".to_string(),
        stats.rdh_stats().rdhs_filtered().to_string(),
        None,
    ));
    // If filtering, the HBFs seen is from the filtered RDHs
    filtered_stats.push(StatSummary::new(
        "HBFs".to_string(),
        stats.rdh_stats().hbfs_seen().to_string(),
        None,
    ));

    filtered_stats.push(summerize_data_size(
        stats.rdh_stats().rdhs_filtered(),
        stats.rdh_stats().payload_size(),
    ));

    if let Some(filter_target) = filter_target {
        let filtered_target = match filter_target {
            FilterTarget::Link(link_id) => {
                summerize_filtered_links(link_id, stats.rdh_stats().links_as_slice())
            }
            FilterTarget::Fee(fee_id) => {
                summerize_filtered_fee_ids(fee_id, stats.rdh_stats().fee_ids_as_slice())
            }
            FilterTarget::ItsLayerStave(fee_id_no_link) => summerize_filtered_its_layer_staves(
                fee_id_no_link,
                stats.rdh_stats().layer_staves_as_slice(),
            ),
        };
        filtered_stats.push(filtered_target);
    }

    if filter_target.is_some_and(|target| !matches!(target, FilterTarget::ItsLayerStave(_))) {
        // Check if the observed system ID is ITS
        if matches!(stats.rdh_stats().system_id(), Some(SystemId::ITS)) {
            // If no filtering, the layers and staves seen is from the total RDHs
            filtered_stats.push(summerize_layers_staves_seen(
                stats.rdh_stats().layer_staves_as_slice(),
                stats.staves_with_errors_as_slice(),
            ));
        }
    }

    filtered_stats
}

// Helper function that adds the ALPIDE stats to the report
fn add_alpide_stats_to_report(report: &mut Report, alpide_stats: &AlpideStats) {
    let mut alpide_stat: Vec<StatSummary> = Vec::new();

    let readout_flags = alpide_stats.readout_flags();

    let chip_trailer_seen_str = readout_flags.chip_trailers_seen().to_string();

    // The chip trailers seen will be the largest number, use the length of
    // the string representation to pad/align the numbers of the readout flags
    let max_num_len = chip_trailer_seen_str.len();

    alpide_stat.push(StatSummary::new(
        "Chip Trailers seen".to_string(),
        chip_trailer_seen_str,
        None,
    ));

    alpide_stat.push(StatSummary::new(
        "Busy Violations".to_string(),
        format!("{:>max_num_len$}", readout_flags.busy_violations()),
        None,
    ));

    alpide_stat.push(StatSummary::new(
        "Data Overrun".to_string(),
        format!("{:>max_num_len$}", readout_flags.data_overrun()),
        None,
    ));

    alpide_stat.push(StatSummary::new(
        "Transmission in Fatal".to_string(),
        format!("{:>max_num_len$}", readout_flags.transmission_in_fatal()),
        None,
    ));

    alpide_stat.push(StatSummary::new(
        "Flushed Incomplete".to_string(),
        format!("{:>max_num_len$}", readout_flags.flushed_incomplete()),
        None,
    ));
    alpide_stat.push(StatSummary::new(
        "Strobe Extended".to_string(),
        format!("{:>max_num_len$}", readout_flags.strobe_extended()),
        None,
    ));
    alpide_stat.push(StatSummary::new(
        "Busy Transitions".to_string(),
        format!("{:>max_num_len$}", readout_flags.busy_transitions()),
        None,
    ));

    report.add_alpide_stats(tabled::Table::new(alpide_stat));
}

// Helper function that adds the detected attributes to the report
fn add_detected_attributes_to_report(report: &mut Report, rdh_stats: &RdhStats) {
    report.add_detected_attribute(
        "RDH Version".to_string(),
        rdh_stats.rdh_version().to_string(),
    );

    report.add_detected_attribute(
        "Data Format".to_string(),
        rdh_stats.data_format().to_string(),
    );
    report.add_detected_attribute(
        "System ID".to_string(),
        // If no system ID is found, something is wrong, set it to "none" in red.
        match rdh_stats.system_id() {
            Some(sys_id) => sys_id.to_string(),
            None => String::from("none").red().to_string(),
        }, // Default to TST for unit tests where no RDHs are seen
    );
}
