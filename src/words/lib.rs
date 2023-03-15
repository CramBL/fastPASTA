use super::{
    data_words::DataWord,
    rdh::{Rdh0, Rdh1, Rdh2, Rdh3},
    rdh_cru::RdhCRU,
};

/// This is the trait that all RDH `subwords` words must implement
/// It is used to:
/// * pretty printing to stdout
/// * deserialize the GBT words from the binary file
pub trait RdhSubWord: std::fmt::Debug + PartialEq + Sized + std::fmt::Display {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
}

pub trait RDH:
    PartialEq + Sized + ByteSlice + std::fmt::Display + std::fmt::Debug + Sync + Send
{
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn load_from_rdh0<T: std::io::Read>(reader: &mut T, rdh0: Rdh0)
        -> Result<Self, std::io::Error>;
    fn version(&self) -> u8;
    fn rdh0(&self) -> &Rdh0;
    fn rdh1(&self) -> &Rdh1;
    fn rdh2(&self) -> &Rdh2;
    fn rdh3(&self) -> &Rdh3;
    fn link_id(&self) -> u8;
    /// Returns the size of the payload in bytes
    /// This is EXCLUDING the size of the RDH
    fn payload_size(&self) -> u16;
    fn offset_to_next(&self) -> u16;
    fn stop_bit(&self) -> u8;
    fn pages_counter(&self) -> u16;
    fn data_format(&self) -> u8;
    fn is_hba(&self) -> bool;
    fn fee_id(&self) -> u16;
    fn cru_id(&self) -> u16;
    fn dw(&self) -> u8;
}

/// This trait is used to convert a struct to a byte slice
/// All structs that are used to represent a full GBT word (not sub RDH words) must implement this trait
pub trait ByteSlice {
    #[inline]
    fn to_byte_slice(&self) -> &[u8]
    where
        Self: Sized,
    {
        unsafe { any_as_u8_slice(self) }
    }
}

impl<Version> ByteSlice for RdhCRU<Version> {}
impl<T> ByteSlice for T where T: DataWord {}
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
