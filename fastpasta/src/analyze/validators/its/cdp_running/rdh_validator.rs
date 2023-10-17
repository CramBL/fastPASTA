//! Contains the [ItsRdhValidator] that keeps track of the current [RDH]
//! and to perform additional checks on an [RDH] which
//! are only applicable because the payload is from ITS.
//!
//! also Allows borrowing the [RDH]

use alice_protocol_reader::rdh::RDH;

/// Holds the [RDH] of a given current CDP and performs checks on it depending on the payload
#[derive(Debug)]
pub struct ItsRdhValidator<T: RDH> {
    rdh: Option<T>,
}

impl<T: RDH> ItsRdhValidator<T> {
    /// Initialize an [ItsRdhValidator] for a reference to an [RDH]
    ///
    /// This should be the first step of checking a CDP
    pub fn new(rdh: &T) -> Self {
        Self {
            rdh: Some(T::load(&mut rdh.to_byte_slice()).unwrap()),
        }
    }

    /// Obtain a reference to the current [RDH] of the current CDP
    ///
    /// Panics if the validator was not initialized with an [RDH]
    pub fn rdh(&self) -> &T {
        self.rdh.as_ref().expect(
            "Attempted to borrow the current RDH before properly initializing the ItsRdhValidator",
        )
    }

    /// Checks RDH stop_bit and pages_counter when a DDW0 is observed
    pub fn check_at_ddw0(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::<String>::new();
        if self.rdh.as_ref().unwrap().stop_bit() != 1 {
            errors.push("[E110] DDW0 observed but RDH stop bit is not 1".into());
        }

        if self.rdh.as_ref().unwrap().pages_counter() == 0 {
            errors.push("[E111] DDW0 observed but RDH page counter is 0".into());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Checks RDH stop_bit when an initial IHW is observed (not IHW during continuation)
    ///
    /// An initial IHW can appear several times in a CDP so page counter checks are not applicable
    pub fn check_at_initial_ihw(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::<String>::new();

        if self.rdh.as_ref().unwrap().stop_bit() != 0 {
            errors.push("[E12] IHW observed but RDH stop bit is not 0".into());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl<T: RDH> Default for ItsRdhValidator<T> {
    fn default() -> Self {
        Self {
            rdh: Default::default(),
        }
    }
}
