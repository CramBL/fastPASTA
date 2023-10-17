//! Contains the [CdpTracker] that tracks state and memory position in a given CDP
//!
//!
//!

use alice_protocol_reader::rdh::RDH;

#[derive(Debug, Default)]
pub struct CdpTracker {
    payload_mem_pos: u64,
    gbt_word_counter: u16,
    gbt_word_padding_size_bytes: u8,
    is_start_of_data: bool, // Flag used to indicate start of new CDP data where a CDW is valid
}

impl CdpTracker {
    pub fn new(rdh: &impl RDH, rdh_mem_pos: u64) -> Self {
        Self {
            payload_mem_pos: rdh_mem_pos + 64,
            gbt_word_counter: 0,
            gbt_word_padding_size_bytes: if rdh.data_format() == 0 {
                6 // Data format 0
            } else {
                0 // Data format 2
            },
            is_start_of_data: true,
        }
    }

    /// Returns if the payload position is the beginning of the payload.
    ///
    /// CDWs are only valid at the start of payloads.
    pub fn start_of_data(&self) -> bool {
        self.is_start_of_data
    }

    /// Report to the tracker that data has been seen in the current CDP
    ///
    /// If any data words are seen, it's no longer start of data
    pub fn set_data_seen(&mut self) {
        self.is_start_of_data = false;
    }

    /// Returns the current position in the memory of the current word.
    ///
    /// It is calculated as follows:
    ///  * Current payload position is the first byte after the current RDH.
    ///
    /// The gbt word position relative to the current payload is then:
    ///  * `gbt_word_mem_size_bytes` = 10 + `gbt_word_padding_size_bytes`
    ///  * `gbt_word_index` = `gbt_word_counter` - 1
    ///  * `relative_mem_pos` = `gbt_word_index` * `gbt_word_mem_size_bytes`
    ///
    /// The absolute position in the memory is then:
    ///
    /// * `gbt_word_mem_pos` = `payload_mem_pos` + `relative_mem_pos`
    #[inline]
    pub fn current_word_mem_pos(&self) -> u64 {
        let gbt_word_memory_size_bytes: u64 = 10 + self.gbt_word_padding_size_bytes as u64;
        let gbt_word_index = (self.gbt_word_counter - 1) as u64; // -1 as it is zero indexed
        let relative_mem_pos = gbt_word_index * gbt_word_memory_size_bytes;
        relative_mem_pos + self.payload_mem_pos
    }

    /// Increment the GBT word counter when a new GBT word is being checked.
    #[inline]
    pub fn incr_word_count(&mut self) {
        self.gbt_word_counter += 1;
    }
}
