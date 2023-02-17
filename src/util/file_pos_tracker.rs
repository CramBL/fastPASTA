pub struct FilePosTracker {
    pub offset_next: i64,
    pub memory_address_bytes: u64,
    pub next_payload_size: usize,
    rdh_cru_size_bytes: u64,
}
impl FilePosTracker {
    pub fn new() -> Self {
        FilePosTracker {
            offset_next: 0,
            memory_address_bytes: 0,
            next_payload_size: 0,
            rdh_cru_size_bytes: 64, // RDH size in bytes
        }
    }
    pub fn next(&mut self, rdh_offset: u64) -> i64 {
        self.offset_next = (rdh_offset - self.rdh_cru_size_bytes) as i64;
        self.memory_address_bytes += rdh_offset;
        self.offset_next
    }

    pub fn update_next_payload_size(&mut self, payload_size: usize) {
        self.next_payload_size = payload_size;
    }

    pub fn next_payload_size(&self) -> usize {
        self.next_payload_size
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_file_tracker() {
        let mut file_tracker = FilePosTracker::new();
        assert_eq!(file_tracker.offset_next, 0);
        assert_eq!(file_tracker.memory_address_bytes, 0);
        assert_eq!(file_tracker.next(64), 0);
        assert_eq!(file_tracker.offset_next, 0);
        assert_eq!(file_tracker.memory_address_bytes, 64);
        assert_eq!(file_tracker.next(64), 0);
        assert_eq!(file_tracker.offset_next, 0);
        assert_eq!(file_tracker.memory_address_bytes, 128);
    }
}
