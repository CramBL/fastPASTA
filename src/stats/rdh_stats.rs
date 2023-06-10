//! Contains the [RdhStats] struct, that holds stats extracted from the RDHs of the raw data

use itertools::Itertools;

use super::lib::SystemId;

/// Stores stats extracted from the RDHs of the raw data.
#[derive(Default)]
pub struct RdhStats {
    /// Total RDHs seen.
    pub rdhs_seen: u64,
    /// Total RDHs filtered.
    pub rdhs_filtered: u64,
    rdh_version: Option<u8>,
    /// Total HBFs seen
    pub hbfs_seen: u32,
    /// Total payload size.
    pub payload_size: u64,
    // Data format observed
    data_format: Option<u8>,
    /// Links observed.
    links: Vec<u8>,
    /// FEE IDs seen
    fee_id: Vec<u16>,
    /// System ID observed in the data
    system_id: Option<SystemId>,
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
    pub fn links_observed(&self) -> &[u8] {
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
    pub fn fee_ids_observed(&self) -> &[u16] {
        self.fee_id.as_slice()
    }

    /// Drains the vector containing the observed FEE IDs
    pub fn consume_fee_ids_observed(&mut self) -> Vec<u16> {
        self.fee_id.drain(..).collect_vec()
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
}
