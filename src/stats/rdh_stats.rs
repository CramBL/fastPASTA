//! Contains the [RdhStats] struct, that holds stats extracted from the RDHs of the raw data

use super::its_stats::ItsStats;
use crate::stats::SystemId;

/// Stores stats extracted from the RDHs of the raw data.
#[derive(Default)]
pub struct RdhStats {
    /// Total RDHs seen.
    rdhs_seen: u64,
    /// Total RDHs filtered.
    rdhs_filtered: u64,
    rdh_version: Option<u8>,
    /// Total HBFs seen
    hbfs_seen: u32,
    /// Total payload size.
    payload_size: u64,
    // Data format observed
    data_format: Option<u8>,
    /// Links observed.
    links: Vec<u8>,
    /// FEE IDs seen
    fee_id: Vec<u16>,
    /// System ID observed in the data
    system_id: Option<SystemId>,
    /// First Trigger Type observed in the data.
    /// Indicates the type of run the data is from.
    /// If the data is from the middle of the run, it won't be as informative.
    run_trigger_type: Option<(u32, String)>,
    /// ITS specific stats retrieved from the RDHs
    its_stats: ItsStats,
}

impl RdhStats {
    /// Stores the RDH version for the raw data.
    ///
    /// Can only bet set once. Setting it more than once will panic.
    pub fn record_rdh_version(&mut self, version: u8) {
        if self.rdh_version.is_some() {
            panic!("Cannot set RDH version more than once!")
        } else {
            self.rdh_version = Some(version);
        }
    }

    /// Retrieves the recorded RDH version.
    ///
    /// Panics if the RDH version was not yet set.
    pub fn rdh_version(&self) -> u8 {
        self.rdh_version
            .expect("RDH version has not been recorded!")
    }

    /// Stores the Data format for the raw data.
    ///
    /// Can only bet set once. Setting it more than once will panic.
    pub fn record_data_format(&mut self, data_format: u8) {
        if self.data_format.is_some() {
            panic!("Cannot set Data format more than once!")
        } else {
            self.data_format = Some(data_format);
        }
    }

    /// Retrieves the recorded Data format.
    ///
    /// Panics if the Data format was not yet set.
    pub fn data_format(&self) -> u8 {
        self.data_format.expect("Data format has not been recoded!")
    }

    /// Stores a link id as observed
    ///
    /// Does not check for duplicates.
    pub fn record_link(&mut self, link_id: u8) {
        self.links.push(link_id);
    }

    /// Sorts the vector containing the observed links
    pub fn sort_links_observed(&mut self) {
        self.links.sort();
    }

    /// Returns a borrowed slice of the vector with the observed links
    pub fn links_as_slice(&self) -> &[u8] {
        self.links.as_slice()
    }

    /// Stores an observed FEE ID if not already seen.
    pub fn record_fee_observed(&mut self, fee_id: u16) {
        // Only add if not already seen
        if !self.fee_id.contains(&fee_id) {
            self.fee_id.push(fee_id);
        }
    }

    /// Returns a borrowed slice of the vector with the observed FEE IDs
    pub fn fee_ids_as_slice(&self) -> &[u16] {
        self.fee_id.as_slice()
    }

    /// Stores a System ID as observed.
    ///
    /// Attempting to set it more than once will panic.
    pub fn record_system_id(&mut self, system_id: SystemId) {
        if self.system_id.is_none() {
            self.system_id = Some(system_id);
        } else {
            panic!("Cannot set system ID more than once!")
        }
    }

    /// Retrieves the recorded System ID if it was set.
    pub fn system_id(&self) -> Option<SystemId> {
        self.system_id
    }

    /// Stores the trigger type in the begging of a run as observed.
    ///
    /// Attempting to set it more than once will panic.
    pub fn record_run_trigger_type(&mut self, run_trigger_type: (u32, String)) {
        if self.run_trigger_type.is_none() {
            self.run_trigger_type = Some(run_trigger_type);
        } else {
            panic!("Cannot set Run Trigger Type more than once!")
        }
    }

    /// Returns the Trigger Type from the start of the run
    ///
    /// Panics if it isn't set.
    pub fn run_trigger_type(&mut self) -> (u32, String) {
        self.run_trigger_type
            .take()
            .expect("Run Trigger Type has not been recorded!")
    }

    /// Stores a layer/stave seen in the raw data.
    ///
    /// This is only applicable if the payload is from ITS.
    pub fn record_layer_stave_seen(&mut self, layer_stave: (u8, u8)) {
        self.its_stats.record_layer_stave_seen(layer_stave);
    }

    /// Returns a borrowed slice of a vector containing the layer/staves seen.
    pub fn layer_staves_as_slice(&self) -> &[(u8, u8)] {
        self.its_stats.layer_staves_as_slice()
    }

    pub(super) fn add_payload_size(&mut self, payload_size: u64) {
        self.payload_size += payload_size;
    }

    pub(super) fn payload_size(&self) -> u64 {
        self.payload_size
    }

    pub(super) fn hbfs_seen(&self) -> u32 {
        self.hbfs_seen
    }

    pub(super) fn incr_hbf_seen(&mut self) {
        self.hbfs_seen += 1;
    }

    pub(super) fn incr_rdhs_seen(&mut self) {
        self.rdhs_seen += 1;
    }

    pub(super) fn rdhs_seen(&self) -> u64 {
        self.rdhs_seen
    }

    pub(super) fn incr_rdhs_filtered(&mut self) {
        self.rdhs_filtered += 1;
    }

    pub(super) fn rdhs_filtered(&self) -> u64 {
        self.rdhs_filtered
    }
}
