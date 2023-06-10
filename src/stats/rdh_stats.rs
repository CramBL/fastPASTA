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
}
