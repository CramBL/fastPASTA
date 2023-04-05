//! This module contains all struct definitions for the words that are used in supported data formats.
use super::{
    rdh::{Rdh0, Rdh1, Rdh2, Rdh3},
    rdh_cru::RdhCRU,
};

/// That all [RDH] `subwords` words must implement
///
/// used for:
/// * pretty printing to stdout
/// * deserialize the GBT words from the binary file
pub trait RdhSubWord: std::fmt::Debug + PartialEq + Sized + std::fmt::Display {
    /// Deserializes the GBT word from a byte slice
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
}

/// Trait that all [RDH] words must implement
///
/// used for:
/// * pretty printing to stdout
/// * deserialize the GBT words from the binary file
/// * access the subwords
/// * access the payload
/// * accessing a variety of fields
pub trait RDH:
    PartialEq + Sized + ByteSlice + std::fmt::Display + std::fmt::Debug + Sync + Send
{
    /// Deserializes the GBT word from a byte slice
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    /// Deserializes the GBT word from an [RDH0][Rdh0] and a byte slice containing the rest of the [RDH]
    fn load_from_rdh0<T: std::io::Read>(reader: &mut T, rdh0: Rdh0)
        -> Result<Self, std::io::Error>;
    /// Returns the version of the [RDH].
    fn version(&self) -> u8;
    /// Returns the subword [RDH0][Rdh0] of the [RDH].
    fn rdh0(&self) -> &Rdh0;
    /// Returns the subword [RDH1][Rdh1] of the [RDH].
    fn rdh1(&self) -> &Rdh1;
    /// Returns the subword [RDH2][Rdh2] of the [RDH].
    fn rdh2(&self) -> &Rdh2;
    /// Returns the subword [RDH3][Rdh3] of the [RDH].
    fn rdh3(&self) -> &Rdh3;
    /// Returns the link id of the [RDH].
    fn link_id(&self) -> u8;
    /// Returns the size of the payload in bytes.
    /// This size is EXCLUDING the size of the RDH.
    fn payload_size(&self) -> u16;
    /// Returns the offset to the next [RDH] in bytes.
    fn offset_to_next(&self) -> u16;
    /// Returns the value of the stop bit.
    fn stop_bit(&self) -> u8;
    /// Returns the value of the page counter.
    fn pages_counter(&self) -> u16;
    /// Returns the value of the data format.
    fn data_format(&self) -> u8;
    /// Returns the value of the trigger type.
    fn trigger_type(&self) -> u32;
    /// Returns the value of the FEE ID.
    fn fee_id(&self) -> u16;
    /// Returns the value of the CRU ID.
    fn cru_id(&self) -> u16;
    /// Returns the value of the DW.
    fn dw(&self) -> u8;
    /// Returns the value of the packet counter.
    fn packet_counter(&self) -> u8;
}

/// Trait used to convert a struct to a byte slice.
/// All structs that are used to represent a full GBT word (not sub RDH words) must implement this trait.
pub trait ByteSlice {
    /// Returns a borrowed byte slice of the struct.
    #[inline]
    fn to_byte_slice(&self) -> &[u8]
    where
        Self: Sized,
    {
        unsafe { any_as_u8_slice(self) }
    }
}

/// Auto implement [ByteSlice] for the following structs.
impl<Version> ByteSlice for RdhCRU<Version> {}
impl ByteSlice for super::status_words::Ihw {}
impl ByteSlice for super::status_words::Tdh {}
impl ByteSlice for super::status_words::Cdw {}
impl ByteSlice for super::status_words::Tdt {}
impl ByteSlice for super::status_words::Ddw0 {}

/// # Safety
/// This function can only be used to serialize a struct if it has the #[repr(packed)] attribute
/// If there's any padding on T, it is UNITIALIZED MEMORY and therefor UNDEFINED BEHAVIOR!
#[inline]
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    // Create read-only reference to T as a byte slice, safe as long as no padding bytes are read
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

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
