//! Contains the [ItsStats] struct which stores ITS specific data observed in the raw data
use serde::{Deserialize, Serialize};
pub mod alpide_stats;

/// Stores ITS specific data observed through RDHs
#[derive(Default, PartialEq, Debug, Serialize, Deserialize)]
pub struct ItsStats {
    /// Holds a layer/stave combinations observed in the raw data
    layer_staves_seen: Vec<(u8, u8)>,
}

impl ItsStats {
    /// Record a layer/stave observed in the data.
    ///
    /// Does not store duplicates.
    pub fn record_layer_stave_seen(&mut self, layer_stave: (u8, u8)) {
        if !self.layer_staves_seen.contains(&layer_stave) {
            self.layer_staves_seen.push(layer_stave);
        }
    }

    /// Returns a borrowed slice of the vector containing the layer/staves seen.
    pub fn layer_staves_as_slice(&self) -> &[(u8, u8)] {
        &self.layer_staves_seen
    }

    pub(super) fn validate_other(&self, other: &Self) -> Result<(), Vec<String>> {
        // Do this (syntax) to ensure that adding a new field to the struct doesn't break the validation
        // If a new field is added, this will fail to compile, before explicitly adding the new field to this instantiation
        let other = Self {
            layer_staves_seen: other.layer_staves_seen.clone(),
        };
        self.validate_fields(&other)
    }
    // Implementation of the `validate_fields` macro
    // Remember to add new fields here as well!
    crate::validate_fields!(ItsStats, layer_staves_seen);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_consistency() {
        let mut its_stats = ItsStats::default();
        its_stats.record_layer_stave_seen((1, 2));
        its_stats.record_layer_stave_seen((3, 4));
        its_stats.record_layer_stave_seen((5, 6));

        // JSON
        let its_stats_ser_json = serde_json::to_string(&its_stats).unwrap();
        let its_stats_de_json: ItsStats = serde_json::from_str(&its_stats_ser_json).unwrap();
        assert_eq!(its_stats, its_stats_de_json);

        // TOML
        let its_stats_ser_toml = toml::to_string(&its_stats).unwrap();
        let its_stats_de_toml: ItsStats = toml::from_str(&its_stats_ser_toml).unwrap();
        assert_eq!(its_stats, its_stats_de_toml);
    }
}
