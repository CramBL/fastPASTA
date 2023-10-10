use crate::{
    stats::StatType,
    words::its::status_words::{Cdw, Ddw0, Ihw, Tdh, Tdt},
};

use super::status_word::StatusWordSanityChecker;

/// Helper function to format and report an error in ITS protocol
///
/// Takes in the error string slice and the word slice
/// Adds the memory position to the error string
/// Sends the error to the stats channel
#[inline]
pub(super) fn report_error(
    mem_pos: u64,
    err: &str,
    word_slice: &[u8],
    sender: &flume::Sender<StatType>,
) {
    sender
            .send(StatType::Error(format!(
                "{mem_pos:#X}: {err} [{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}]",
                word_slice[0],
                word_slice[1],
                word_slice[2],
                word_slice[3],
                word_slice[4],
                word_slice[5],
                word_slice[6],
                word_slice[7],
                word_slice[8],
                word_slice[9],
                            ).into()))
            .expect("Failed to send error to stats channel");
}

#[derive(Default)]
struct TdhBuffer {
    current_tdh: Option<Tdh>,
    previous_tdh: Option<Tdh>,
    // Last TDH with internal trigger bit set
    previous_tdh_with_internal_set: Option<Tdh>,
}

impl TdhBuffer {
    pub(crate) fn replace(&mut self, tdh: Tdh) {
        let old_tdh = self.current_tdh.replace(tdh);
        // If the previous TDH had internal trigger set, set it to previous internal TDH.
        if old_tdh.is_some_and(|old| old.internal_trigger() == 1) {
            self.previous_tdh_with_internal_set = old_tdh;
        }
        self.previous_tdh = old_tdh;
    }

    pub(crate) fn current_tdh(&self) -> Option<&Tdh> {
        self.current_tdh.as_ref()
    }

    pub(crate) fn previous_tdh(&self) -> Option<&Tdh> {
        self.previous_tdh.as_ref()
    }

    pub(crate) fn previous_tdh_with_internal_trg(&self) -> Option<&Tdh> {
        self.previous_tdh_with_internal_set.as_ref()
    }
}

pub struct StatusWordContainer {
    sanity_checker: StatusWordSanityChecker,
    ihw: Option<Ihw>,
    tdhs: TdhBuffer,
    tdt: Option<Tdt>,
    ddw0: Option<Ddw0>,
    cdw: Option<Cdw>,
}

impl StatusWordContainer {
    pub const fn new_const() -> Self {
        Self {
            sanity_checker: StatusWordSanityChecker::new(),
            ihw: None,
            tdhs: TdhBuffer {
                current_tdh: None,
                previous_tdh: None,
                previous_tdh_with_internal_set: None,
            },
            tdt: None,
            ddw0: None,
            cdw: None,
        }
    }

    pub fn sanity_check_tdh(&self, tdh: &Tdh) -> Result<(), String> {
        self.sanity_checker.check_tdh(tdh)
    }

    pub fn replace_tdh(&mut self, tdh: Tdh) {
        self.tdhs.replace(tdh);
    }

    pub fn tdh(&self) -> Option<&Tdh> {
        self.tdhs.current_tdh()
    }

    pub fn prv_tdh(&self) -> Option<&Tdh> {
        self.tdhs.previous_tdh()
    }

    pub fn tdh_previous_with_internal_trg(&self) -> Option<&Tdh> {
        self.tdhs.previous_tdh_with_internal_trg()
    }

    /// Checks if argument is a valid [TDT][Tdt] status word.
    pub fn sanity_check_tdt(&self, tdt: &Tdt) -> Result<(), String> {
        self.sanity_checker.check_tdt(tdt)
    }

    pub fn replace_tdt(&mut self, tdt: Tdt) {
        self.tdt = Some(tdt);
    }
    pub fn tdt(&self) -> Option<&Tdt> {
        self.tdt.as_ref()
    }

    /// Checks if argument is a valid [IHW][Ihw] status word.
    pub fn sanity_check_ihw(&self, ihw: &Ihw) -> Result<(), String> {
        self.sanity_checker.check_ihw(ihw)
    }

    pub fn replace_ihw(&mut self, ihw: Ihw) {
        self.ihw = Some(ihw);
    }

    pub fn ihw(&self) -> Option<&Ihw> {
        self.ihw.as_ref()
    }

    /// Checks if argument is a valid [DDW0][Ddw0] status word.
    pub fn sanity_check_ddw0(&self, ddw0: &Ddw0) -> Result<(), String> {
        self.sanity_checker.check_ddw0(ddw0)
    }

    pub fn replace_ddw(&mut self, ddw0: Ddw0) {
        self.ddw0 = Some(ddw0);
    }

    pub fn ddw(&self) -> Option<&Ddw0> {
        self.ddw0.as_ref()
    }

    pub fn replace_cdw(&mut self, cdw: Cdw) {
        self.cdw = Some(cdw);
    }

    pub fn cdw(&self) -> Option<&Cdw> {
        self.cdw.as_ref()
    }
}
