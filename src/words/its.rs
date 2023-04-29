//! Data definitions for ITS payload words
pub mod data_words;
pub mod status_words;

// Utility functions to extract information from the FeeId
/// Extracts stave_number from 6 LSB \[5:0\]
pub fn stave_number_from_feeid(fee_id: u16) -> u8 {
    let stave_number_mask: u16 = 0b11_1111;
    (fee_id & stave_number_mask) as u8
}
/// Extracts layer number from 3 bits \[14:12\]
pub fn layer_from_feeid(fee_id: u16) -> u8 {
    // Extract layer from 3 bits 14:12
    let layer_mask: u16 = 0b0111;
    let layer_lsb_idx: u8 = 12;
    ((fee_id >> layer_lsb_idx) & layer_mask) as u8
}
