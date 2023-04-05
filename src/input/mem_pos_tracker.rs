//! Contains the MemPosTracker struct that aids in tracking the memory position in the input data.

/// Tracks the position by the value of RDH offsets received in the next() function.
pub struct MemPosTracker {
    /// The memory address in bytes of the current RDH.
    pub memory_address_bytes: u64,
    offset_next: i64,
    rdh_cru_size_bytes: u64,
}

impl Default for MemPosTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl MemPosTracker {
    /// Create a new MemPosTracker.
    pub fn new() -> Self {
        MemPosTracker {
            offset_next: 0,
            memory_address_bytes: 0,
            rdh_cru_size_bytes: 64, // RDH size in bytes
        }
    }
    /// Get the relative offset of the next RDH.
    ///
    /// The offset is relative to the current RDH, and uses the RDH size as a base.
    /// Takes the offset of the next RDH in bytes.
    pub fn next(&mut self, rdh_offset: u64) -> i64 {
        debug_assert!(
            rdh_offset >= self.rdh_cru_size_bytes,
            "RDH offset is smaller than RDH size: {} < {}",
            rdh_offset,
            self.rdh_cru_size_bytes
        );
        self.offset_next = (rdh_offset - self.rdh_cru_size_bytes) as i64;
        self.memory_address_bytes += rdh_offset;
        self.offset_next
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_file_tracker() {
        let mut file_tracker = MemPosTracker::new();
        assert_eq!(file_tracker.offset_next, 0);
        assert_eq!(file_tracker.memory_address_bytes, 0);
        assert_eq!(file_tracker.next(64), 0);
        assert_eq!(file_tracker.offset_next, 0);
        assert_eq!(file_tracker.memory_address_bytes, 64);
        assert_eq!(file_tracker.next(64), 0);
        assert_eq!(file_tracker.offset_next, 0);
        assert_eq!(file_tracker.memory_address_bytes, 128);
    }
    #[test]
    fn test_file_tracker_default() {
        let mut file_tracker = MemPosTracker::default();
        assert_eq!(file_tracker.offset_next, 0);
        assert_eq!(file_tracker.memory_address_bytes, 0);
        assert_eq!(file_tracker.next(64), 0);
        assert_eq!(file_tracker.offset_next, 0);
        assert_eq!(file_tracker.memory_address_bytes, 64);
        assert_eq!(file_tracker.next(64), 0);
        assert_eq!(file_tracker.offset_next, 0);
        assert_eq!(file_tracker.memory_address_bytes, 128);
    }
    #[test]
    #[should_panic]
    fn test_panic_file_tracker_default() {
        let mut file_tracker = MemPosTracker::default();
        assert_eq!(file_tracker.offset_next, 0);
        assert_eq!(file_tracker.memory_address_bytes, 0);
        assert_eq!(file_tracker.next(64), 0);
        assert_eq!(file_tracker.offset_next, 0);
        assert_eq!(file_tracker.memory_address_bytes, 64);
        assert_eq!(file_tracker.next(64), 0);
        assert_eq!(file_tracker.offset_next, 0);
        assert_eq!(file_tracker.memory_address_bytes, 128);
        // This should panic
        file_tracker.next(63);
    }
}
