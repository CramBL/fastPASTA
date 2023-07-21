//! Contains the [ItsStats] struct which stores ITS specific data observed in the raw data

pub mod alpide_stats;

/// Stores ITS specific data observed through RDHs
#[derive(Default)]
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
}
