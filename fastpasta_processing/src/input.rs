//! All functionality related to reading data from a file or stdin, tracking memory offset and filtering data.

mod bufreader_wrapper;
pub mod config;
pub mod data_wrapper;
mod input_scanner;
pub mod lib;
mod mem_pos_tracker;
pub mod prelude;
pub mod rdh;
pub mod stdin_reader;
