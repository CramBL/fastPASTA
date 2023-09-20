use owo_colors::OwoColorize;

use super::report::StatSummary;
use super::stat_format_utils::format_data_size;
use super::stat_format_utils::format_layers_and_staves;
use crate::words::its::layer_from_feeid;
use crate::words::its::stave_number_from_feeid;
use alice_protocol_reader::prelude::RDH_CRU_SIZE_BYTES;

/// Helper functions to format the summary of filtered link ID
pub(crate) fn summerize_filtered_links(link_to_filter: u8, links_observed: &[u8]) -> StatSummary {
    let mut filtered_links_stat = StatSummary::new("Link ID".to_string(), "".to_string(), None);
    // Format links that were filtered, separated by commas
    if links_observed.contains(&link_to_filter) {
        filtered_links_stat.value = link_to_filter.to_string();
    } else {
        filtered_links_stat.value = "none".red().to_string();
        filtered_links_stat.notes = format!("not found: {link_to_filter}").red().to_string();
    }
    filtered_links_stat
}

/// Helper functions to format the summary of filtered FEE ID
pub(crate) fn summerize_filtered_fee_ids(fee_id: u16, fee_ids_seen: &[u16]) -> StatSummary {
    let mut filtered_feeid_stat = StatSummary::new("FEE ID".to_string(), "".to_string(), None);

    if fee_ids_seen.contains(&fee_id) {
        filtered_feeid_stat.value = fee_id.to_string();
    } else {
        filtered_feeid_stat.value = "none".red().to_string();
        filtered_feeid_stat.notes = format!("not found: {fee_id}").red().to_string();
    }
    filtered_feeid_stat
}

/// Helper functions to format the summary of filtered ITS layer and stave
pub(crate) fn summerize_filtered_its_layer_staves(
    fee_id_no_link: u16,
    layers_staves_seen: &[(u8, u8)],
) -> StatSummary {
    let mut filtered_feeid_stat = StatSummary::new("ITS stave".to_string(), "".to_string(), None);
    let layer = layer_from_feeid(fee_id_no_link);
    let stave = stave_number_from_feeid(fee_id_no_link);
    if layers_staves_seen.contains(&(layer, stave)) {
        filtered_feeid_stat.value = format!("L{layer}_{stave}");
    } else {
        filtered_feeid_stat.value = "none".red().to_string();
        filtered_feeid_stat.notes = format!("not found: L{layer}_{stave}").red().to_string();
    }
    filtered_feeid_stat
}

pub(crate) fn summerize_layers_staves_seen(
    layers_staves_seen: &[(u8, u8)],
    staves_with_errors: Option<&[(u8, u8)]>,
) -> StatSummary {
    let with_errors = if let Some(staves_with_errors) = staves_with_errors {
        staves_with_errors.to_owned()
    } else {
        // If it's none, make a new empty vector
        std::vec::Vec::new()
    };
    StatSummary::new(
        "Layers/Staves".to_string(),
        format_layers_and_staves(layers_staves_seen.to_owned(), with_errors),
        None,
    )
}

pub(crate) fn summerize_data_size(rdh_count: u64, payload_size: u64) -> StatSummary {
    let rdh_data_size = rdh_count * RDH_CRU_SIZE_BYTES as u64;
    if rdh_data_size == 0 {
        StatSummary::new("Data size".to_string(), format_data_size(0), None)
    } else {
        StatSummary::new(
            "Data size".to_string(),
            format_data_size(rdh_data_size + payload_size),
            Some(format!(
                "RDHs:     {rdhs_size}\nPayloads: {payloads_size}",
                rdhs_size = format_data_size(rdh_data_size),
                payloads_size = format_data_size(payload_size)
            )),
        )
    }
}
