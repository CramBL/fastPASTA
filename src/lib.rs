pub mod data_write;
pub mod input;
pub mod stats;
pub mod util;
pub mod validators;
pub mod words;

// Larger capacity means less overhead, but more memory usage
// Too small capacity will cause the producer thread to block
// Too large capacity will cause down stream consumers to block
pub const CHANNEL_CDP_CAPACITY: usize = 100;
