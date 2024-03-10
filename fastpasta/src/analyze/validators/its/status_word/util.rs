//! Utility for analyzing status words

use crate::util::*;

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

/// Holds status words and allows accessing and replacing them continuously
pub struct StatusWordContainer {
    ihw: Option<Ihw>,
    tdhs: TdhBuffer,
    tdt: Option<Tdt>,
    ddw0: Option<Ddw0>,
    cdw: Option<Cdw>,
}

impl StatusWordContainer {
    /// Create a const instance of [StatusWordContainer]
    pub const fn new_const() -> Self {
        Self {
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

    /// Perform sanity check on a [TDH][Tdh]
    pub fn sanity_check_tdh(&self, tdh: &Tdh) -> Result<(), String> {
        StatusWordSanityChecker::check_tdh(tdh)
    }

    /// Replace the stored [TDH][Tdh] with a new [TDH][Tdh]
    pub fn replace_tdh(&mut self, tdh: Tdh) {
        self.tdhs.replace(tdh);
    }

    /// Get a reference to the current [TDH][Tdh].
    /// Returns [None] if the current [TDH][Tdh] has not been set yet (no [TDH][Tdh] seen in the data yet).
    pub fn tdh(&self) -> Option<&Tdh> {
        self.tdhs.current_tdh()
    }

    /// Get a reference to the previous [TDH][Tdh].
    /// Returns [None] if the previous [TDH][Tdh] has not been set yet (only one [TDH][Tdh] has been seen).
    pub fn prv_tdh(&self) -> Option<&Tdh> {
        self.tdhs.previous_tdh()
    }

    /// Get a reference to the previos [TDH][Tdh] that has the internal trigger field set.
    /// [None] if no previous [TDH][Tdh] with internal trigger set was seen yet.
    pub fn tdh_previous_with_internal_trg(&self) -> Option<&Tdh> {
        self.tdhs.previous_tdh_with_internal_trg()
    }

    /// Checks if argument is a valid [TDT][Tdt] status word.
    pub fn sanity_check_tdt(&self, tdt: &Tdt) -> Result<(), String> {
        StatusWordSanityChecker::check_tdt(tdt)
    }

    /// Replace the stored [TDT][Tdt] with a new [TDT][Tdt]
    pub fn replace_tdt(&mut self, tdt: Tdt) {
        self.tdt = Some(tdt);
    }

    /// Get a reference to the stored [TDT][Tdt].
    /// Returns [None] if no [TDT][Tdt] was set (seen in the data) yet.
    pub fn tdt(&self) -> Option<&Tdt> {
        self.tdt.as_ref()
    }

    /// Checks if argument is a valid [IHW][Ihw] status word.
    pub fn sanity_check_ihw(&self, ihw: &Ihw) -> Result<(), String> {
        StatusWordSanityChecker::check_ihw(ihw)
    }

    /// Replace the stored [IHW][Ihw] with a new [IHW][Ihw]
    pub fn replace_ihw(&mut self, ihw: Ihw) {
        self.ihw = Some(ihw);
    }

    /// Get a reference to the stored [IHW][Ihw].
    /// Returns [None] if no [IHW][Ihw] was set (seen in the data) yet.
    pub fn ihw(&self) -> Option<&Ihw> {
        self.ihw.as_ref()
    }

    /// Checks if argument is a valid [DDW0][Ddw0] status word.
    pub fn sanity_check_ddw0(&self, ddw0: &Ddw0) -> Result<(), String> {
        StatusWordSanityChecker::check_ddw0(ddw0)
    }

    /// Replace the stored [DDW][Ddw0] with a new [DDW][Ddw0]
    pub fn replace_ddw(&mut self, ddw0: Ddw0) {
        self.ddw0 = Some(ddw0);
    }

    /// Get a reference to the stored [DDW][Ddw0].
    /// Returns [None] if no [DDW][Ddw0] was set (seen in the data) yet.
    pub fn ddw(&self) -> Option<&Ddw0> {
        self.ddw0.as_ref()
    }

    /// Replace the stored [CDW][Cdw] with a new [CDW][Cdw]
    pub fn replace_cdw(&mut self, cdw: Cdw) {
        self.cdw = Some(cdw);
    }

    /// Get a reference to the stored [CDW][Cdw].
    /// Returns [None] if no [CDW][Cdw] was set (seen in the data) yet.
    pub fn cdw(&self) -> Option<&Cdw> {
        self.cdw.as_ref()
    }
}
