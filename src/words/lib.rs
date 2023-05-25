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
pub trait RdhSubWord: Sized + PartialEq + std::fmt::Debug + std::fmt::Display {
    /// Deserializes the GBT word from a byte slice
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>;
}

/// Trait that all [RDH] words must implement
///
/// used for:
/// * pretty printing to stdout
/// * deserialize the GBT words from the binary file
/// * access the subwords
/// * access the payload
/// * accessing a variety of fields
pub trait RDH: PartialEq + Sized + std::fmt::Display + std::fmt::Debug + Sync + Send
where
    Self: SerdeRdh,
{
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

/// Trait to Serialize/Deserialise (serde) [RDH] words.
pub trait SerdeRdh: Send + Sync + Sized
where
    Self: ByteSlice,
{
    /// Deserializes the GBT word from a byte slice
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>;
    /// Deserializes the GBT word from an [RDH0][Rdh0] and a byte slice containing the rest of the [RDH]
    fn load_from_rdh0<T: std::io::Read>(reader: &mut T, rdh0: Rdh0)
        -> Result<Self, std::io::Error>;
}

/// Trait used to convert a struct to a byte slice.
/// All structs that are used to represent a full GBT word (not sub RDH words) must implement this trait.
pub trait ByteSlice: Sized {
    /// Returns a borrowed byte slice of the struct.
    #[inline]
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { any_as_u8_slice(self) }
    }
}
impl<T> ByteSlice for &T where T: ByteSlice {}
impl<T> ByteSlice for &mut T where T: ByteSlice {}

/// Auto implement [ByteSlice] for the following structs.
impl<Version> ByteSlice for RdhCRU<Version> {}
impl ByteSlice for super::its::status_words::Ihw {}
impl ByteSlice for super::its::status_words::Tdh {}
impl ByteSlice for super::its::status_words::Cdw {}
impl ByteSlice for super::its::status_words::Tdt {}
impl ByteSlice for super::its::status_words::Ddw0 {}

/// # Safety
/// This function can only be used to serialize a struct if it has the #[repr(packed)] attribute
/// If there's any padding on T, it is UNITIALIZED MEMORY and therefor UNDEFINED BEHAVIOR!
#[inline]
unsafe fn any_as_u8_slice<T: Sized>(packed: &T) -> &[u8] {
    use core::{mem::size_of, slice::from_raw_parts};
    // Create read-only reference to T as a byte slice, safe as long as no padding bytes are read
    from_raw_parts((packed as *const T) as *const u8, size_of::<T>())
}
