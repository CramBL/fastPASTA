//! Contains the [InputStatType] enum for which kind of statistics are gathered, and the [Stats] struct for tracking and reporting statistics about the input data.

#[allow(variant_size_differences)] // Allow in this case, the string is already a pointer.
#[derive(Debug, Clone, PartialEq)]
/// Possible stats that can be sent to the StatsController.
pub enum InputStatType {
    /// Fatal error, stop processing.
    Fatal(Box<str>),
    /// The first trigger type observed is the type of run the data comes from
    ///
    /// Contains the raw value and the string description summarizing the trigger type
    RunTriggerType(u32),
    /// Record the data format detected.
    DataFormat(u8),
    /// Add a link to the list of links observed.
    LinksObserved(u8),
    /// Record the generic FEE ID
    FeeId(u16),
    /// Increment the total RDHs seen.
    RDHSeen(u16),
    /// Increment the total RDHs filtered.
    RDHFiltered(u16),
    /// Increment the total payload size.
    PayloadSize(u32),
    /// The first system ID observed is the basis for the rest of processing
    SystemId(u8),
}

/// Struct for tracking and reporting statistics about the input data.
#[derive(Debug)]
pub struct Stats {
    reporter: flume::Sender<InputStatType>,
    rdhs_seen: u16,
    rdhs_filtered: u16,
    payload_size_seen: u32,
    unique_links_observed: Vec<u8>,
    unique_feeids_observed: Vec<u16>,
}

impl Stats {
    /// Create a new [Stats] instance.
    pub fn new(reporter: flume::Sender<InputStatType>) -> Self {
        Self {
            reporter,
            rdhs_seen: 0,
            rdhs_filtered: 0,
            payload_size_seen: 0,
            unique_links_observed: Vec::new(),
            unique_feeids_observed: Vec::new(),
        }
    }

    /// Attempt to add a link id to the observed links (is only added if not already present in the list).
    pub fn try_add_link(&mut self, link: u8) {
        if !self.unique_links_observed.contains(&link) {
            self.unique_links_observed.push(link);
            self.reporter
                .send(InputStatType::LinksObserved(link))
                .unwrap();
        }
    }

    /// Attempt to add a FEE ID to the observed FEE IDs (is only added if not already present in the list).
    pub fn try_add_fee_id(&mut self, fee_id: u16) {
        if !self.unique_feeids_observed.contains(&fee_id) {
            self.unique_feeids_observed.push(fee_id);
            self.reporter.send(InputStatType::FeeId(fee_id)).unwrap();
        }
    }

    /// Increment the RDH seen counter..
    pub fn rdh_seen(&mut self) {
        self.rdhs_seen += 1;
        if self.rdhs_seen == 1000 {
            self.reporter.send(InputStatType::RDHSeen(1000)).unwrap();
            self.rdhs_seen = 0;
        }
    }

    /// Increment the RDH filtered counter.
    pub fn rdh_filtered(&mut self) {
        self.rdhs_filtered += 1;
        if self.rdhs_filtered == 1000 {
            self.reporter
                .send(InputStatType::RDHFiltered(1000))
                .unwrap();
            self.rdhs_filtered = 0;
        }
    }

    /// Add a payload size to the total payload size seen.
    pub fn add_payload_size(&mut self, payload_size: u16) {
        self.payload_size_seen += payload_size as u32;
        // 10 MB
        if self.payload_size_seen > (10 * 1048576) {
            self.reporter
                .send(InputStatType::PayloadSize(self.payload_size_seen))
                .unwrap();
            self.payload_size_seen = 0;
        }
    }

    /// Flush the stats to the reporter channel (sends all the current stats).
    pub fn flush_stats(&mut self) {
        self.reporter
            .send(InputStatType::RDHSeen(self.rdhs_seen))
            .unwrap();
        self.reporter
            .send(InputStatType::RDHFiltered(self.rdhs_filtered))
            .unwrap();
        self.reporter
            .send(InputStatType::PayloadSize(self.payload_size_seen))
            .unwrap();
    }
}
