//! Contains the Trait [FilterOpt] for all filter options, and the [FilterTarget] enum for the filter target

/// Trait for all filter options set by the user
pub trait FilterOpt {
    /// Link ID to filter by
    fn filter_link(&self) -> Option<u8>;
    /// FEE ID to filter by
    fn filter_fee(&self) -> Option<u16>;
    /// ITS layer & stave to filter by
    fn filter_its_stave(&self) -> Option<u16>;

    /// Get the target of the filter
    fn filter_target(&self) -> Option<FilterTarget> {
        #[allow(clippy::manual_map)] // Clippy is wrong here
        if let Some(link) = self.filter_link() {
            Some(FilterTarget::Link(link))
        } else if let Some(fee) = self.filter_fee() {
            Some(FilterTarget::Fee(fee))
        } else if let Some(its_layer_stave) = self.filter_its_stave() {
            Some(FilterTarget::ItsLayerStave(its_layer_stave))
        } else {
            None
        }
    }

    /// Determine if the filter is enabled
    fn filter_enabled(&self) -> bool {
        self.filter_link().is_some()
            || self.filter_fee().is_some()
            || self.filter_its_stave().is_some()
    }
}

impl<T> FilterOpt for &T
where
    T: FilterOpt,
{
    fn filter_link(&self) -> Option<u8> {
        (*self).filter_link()
    }
    fn filter_fee(&self) -> Option<u16> {
        (*self).filter_fee()
    }
    fn filter_its_stave(&self) -> Option<u16> {
        (*self).filter_its_stave()
    }
}
impl<T> FilterOpt for Box<T>
where
    T: FilterOpt,
{
    fn filter_link(&self) -> Option<u8> {
        (**self).filter_link()
    }
    fn filter_fee(&self) -> Option<u16> {
        (**self).filter_fee()
    }
    fn filter_its_stave(&self) -> Option<u16> {
        (**self).filter_its_stave()
    }
}
impl<T> FilterOpt for std::sync::Arc<T>
where
    T: FilterOpt,
{
    fn filter_link(&self) -> Option<u8> {
        (**self).filter_link()
    }
    fn filter_fee(&self) -> Option<u16> {
        (**self).filter_fee()
    }
    fn filter_its_stave(&self) -> Option<u16> {
        (**self).filter_its_stave()
    }
}

#[derive(Debug, Clone, Copy)]
/// The target of an optional filter on the input data
pub enum FilterTarget {
    /// Filter on the link ID
    Link(u8),
    /// Filter on the FEE ID
    Fee(u16),
    /// Filter on the ITS layer and stave
    ItsLayerStave(u16),
}
