//! Contains the [RdhStats] struct, that holds stats extracted from the RDHs of the raw data

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
    data_format: Option<u8>,
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
}
