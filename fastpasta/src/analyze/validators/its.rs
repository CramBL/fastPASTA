//! # ITS specific payload validation
//!
//! The ITS specific payload validation is facilitated through the [lib::do_payload_checks] function.
//!
//! The [lib::do_payload_checks] function is called from the [LinkValidator](crate::analyze::validators::link_validator::LinkValidator) when the system target is ITS.
//!
//! The [CdpRunningValidator](crate::analyze::validators::its::cdp_running::CdpRunningValidator) is used to validate the payload, and contains all the subvalidators as well.

pub mod alpide;
pub mod cdp_running;
pub mod data_words;
pub mod its_payload_fsm_cont;
pub mod lib;
pub mod status_word;
mod util;
