//! Contains the [RdhStats] struct, that holds stats extracted from the RDHs of the raw data

use super::super::stats_collector::its_stats::ItsStats;
use super::trigger_stats::TriggerStats;
use crate::util::*;

/// Stores stats extracted from the RDHs of the raw data.
#[derive(Default, PartialEq, Debug, Serialize, Deserialize)]
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
    run_trigger_type: Option<(u32, Box<str>)>,
    /// ITS specific stats retrieved from the RDHs
    its_stats: ItsStats,
    /// Stats for the trigger types observed in the data
    trigger_stats: TriggerStats,
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
        self.links.sort_unstable();
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
    pub fn record_run_trigger_type(&mut self, run_trigger_type: (u32, Box<str>)) {
        if self.run_trigger_type.is_none() {
            self.run_trigger_type = Some(run_trigger_type);
        } else {
            panic!("Cannot set Run Trigger Type more than once!")
        }
    }

    /// Returns the Trigger Type from the start of the run
    ///
    /// Panics if it isn't set.
    pub fn run_trigger_type(&self) -> (u32, Box<str>) {
        self.run_trigger_type
            .clone()
            .expect("Run Trigger Type has not been recorded!")
    }

    /// Records trigger type stats
    pub fn record_trigger_type(&mut self, trigger_type: u32) {
        self.trigger_stats.collect_stats(trigger_type);
    }

    /// Returns a borrowed reference to [TriggerStats]
    pub fn trigger_stats(&self) -> &TriggerStats {
        &self.trigger_stats
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

    pub(crate) fn payload_size(&self) -> u64 {
        self.payload_size
    }

    pub(crate) fn hbfs_seen(&self) -> u32 {
        self.hbfs_seen
    }

    pub(super) fn incr_hbf_seen(&mut self) {
        self.hbfs_seen += 1;
    }

    #[allow(dead_code)]
    pub(super) fn incr_rdhs_seen(&mut self) {
        self.rdhs_seen += 1;
    }

    pub(super) fn add_rdhs_seen(&mut self, rdhs_seen: u16) {
        self.rdhs_seen += rdhs_seen as u64;
    }

    pub(crate) fn rdhs_seen(&self) -> u64 {
        self.rdhs_seen
    }

    #[allow(dead_code)]
    pub(super) fn incr_rdhs_filtered(&mut self) {
        self.rdhs_filtered += 1;
    }

    pub(super) fn add_rdhs_filtered(&mut self, rdhs_filtered: u32) {
        self.rdhs_filtered += rdhs_filtered as u64;
    }

    pub(crate) fn rdhs_filtered(&self) -> u64 {
        self.rdhs_filtered
    }

    pub(crate) fn finalize(&mut self) {
        self.sort_links_observed();
    }

    pub(super) fn validate_other(&self, other: &Self) -> Result<(), Vec<String>> {
        let mut errs: Vec<String> = vec![];

        if let Err(mut sub_errs) = self.its_stats.validate_other(&other.its_stats) {
            errs.append(&mut sub_errs);
        }

        if let Err(mut sub_errs) = self.trigger_stats.validate_other(&other.trigger_stats) {
            errs.append(&mut sub_errs);
        }

        // This syntax is used to ensure that a compile error is raised if a
        // new field is added to the struct but not added to the validation here
        // Also add new fields to the `validate_fields` macro!
        let other_top_fields_only = Self {
            rdhs_seen: other.rdhs_seen,
            rdhs_filtered: other.rdhs_filtered,
            rdh_version: other.rdh_version,
            hbfs_seen: other.hbfs_seen,
            payload_size: other.payload_size,
            data_format: other.data_format,
            links: other.links.clone(),
            fee_id: other.fee_id.clone(),
            system_id: other.system_id,
            run_trigger_type: other.run_trigger_type.clone(),
            its_stats: ItsStats::default(), // Validated in previous seperate function
            trigger_stats: TriggerStats::default(), // Validated in seperate function
        };

        if let Err(mut local_top_field_errs) = self.validate_fields(&other_top_fields_only) {
            errs.append(&mut local_top_field_errs);
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }

    // Implementation of the `validate_fields` macro
    // Remember to add new fields here as well!
    crate::validate_fields!(
        RdhStats,
        rdhs_seen,
        rdhs_filtered,
        rdh_version,
        hbfs_seen,
        payload_size,
        data_format,
        links,
        fee_id,
        system_id,
        run_trigger_type
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_serde_consistency() {
        let mut rdh_stats = RdhStats {
            rdhs_seen: 10,
            rdhs_filtered: 0,
            rdh_version: Some(7),
            hbfs_seen: 0,
            payload_size: 0,
            data_format: Some(0),
            links: vec![0, 1, 2, 3, 4, 5, 6],
            fee_id: vec![8, 9, 10, 11, 12, 13, 14],
            system_id: Some(SystemId::MFT),
            run_trigger_type: Some((1, "Test".into())),
            its_stats: ItsStats::default(),
            trigger_stats: TriggerStats::default(),
        };

        rdh_stats.incr_hbf_seen();

        let rdh_stats_ser_json = serde_json::to_string(&rdh_stats).unwrap();
        println!("{}", serde_json::to_string_pretty(&rdh_stats).unwrap());
        let rdh_stats_de_json: RdhStats = serde_json::from_str(&rdh_stats_ser_json).unwrap();

        assert_eq!(rdh_stats, rdh_stats_de_json);

        let rdh_stats_ser_toml = toml::to_string(&rdh_stats).unwrap();
        println!("{}", rdh_stats_ser_toml);
        let rdh_stats_de_toml: RdhStats = toml::from_str(&rdh_stats_ser_toml).unwrap();
        assert_eq!(rdh_stats, rdh_stats_de_toml);
    }
}
