use crate::words::its::status_words::{Cdw, Ddw0, Ihw, Tdh, Tdt};

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

    fn is_previous_internal(&self) -> bool {
        self.previous_tdh
            .is_some_and(|tdh| tdh.internal_trigger() == 1)
    }
}

#[derive(Default)]
pub struct StatusWordContainer {
    ihw: Option<Ihw>,
    tdhs: TdhBuffer,
    tdt: Option<Tdt>,
    ddw0: Option<Ddw0>,
    cdw: Option<Cdw>,
}

impl StatusWordContainer {
    pub fn replace_tdh(&mut self, tdh: Tdh) {
        self.tdhs.replace(tdh);
    }

    pub fn tdh(&self) -> Option<&Tdh> {
        self.tdhs.current_tdh()
    }

    pub fn prv_tdh(&self) -> Option<&Tdh> {
        self.tdhs.previous_tdh()
    }

    pub fn tdh_previous_has_internal_trg(&self) -> bool {
        self.tdhs.is_previous_internal()
    }

    pub fn tdh_previous_with_internal_trg(&self) -> Option<&Tdh> {
        self.tdhs.previous_tdh_with_internal_trg()
    }

    pub fn replace_tdt(&mut self, tdt: Tdt) {
        self.tdt = Some(tdt);
    }
    pub fn tdt(&self) -> Option<&Tdt> {
        self.tdt.as_ref()
    }

    pub fn replace_ihw(&mut self, ihw: Ihw) {
        self.ihw = Some(ihw);
    }

    pub fn ihw(&self) -> Option<&Ihw> {
        self.ihw.as_ref()
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
