//! This module contains all struct definitions for the words that are used in supported data formats.

pub mod rdh0;
pub mod rdh1;
pub mod rdh2;
pub mod rdh3;
pub mod rdh_cru;
pub mod test_data;
pub use rdh0::Rdh0;
use rdh1::Rdh1;
use rdh2::Rdh2;
use rdh3::Rdh3;
pub use rdh_cru::RdhCru;

/// The size of a RDH-CRU word in bytes
pub const RDH_CRU_SIZE_BYTES: u8 = 64;

/// That all [RDH] `subwords` words must implement
///
/// used for:
/// * pretty printing to stdout
/// * deserialize the GBT words from the binary file
pub trait RdhSubword: Sized + PartialEq + std::fmt::Debug + std::fmt::Display {
    /// Deserializes the GBT word from a provided reader
    #[inline]
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error> {
        let raw = macros::load_bytes!(8, reader);
        Self::from_buf(&raw)
    }
    /// Deserializes the GBT word from a byte slice
    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error>;
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
    Self: SerdeRdh + RDH_CRU,
{
}

#[allow(non_camel_case_types)]
/// Trait for accessing fields of a [RDH] word of the RDH CRU flavor.
pub trait RDH_CRU {
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

impl<T> RDH_CRU for &T
where
    T: RDH_CRU,
{
    #[inline]
    fn version(&self) -> u8 {
        (*self).version()
    }

    #[inline]
    fn rdh0(&self) -> &Rdh0 {
        (*self).rdh0()
    }

    #[inline]
    fn rdh1(&self) -> &Rdh1 {
        (*self).rdh1()
    }

    #[inline]
    fn rdh2(&self) -> &Rdh2 {
        (*self).rdh2()
    }

    #[inline]
    fn rdh3(&self) -> &Rdh3 {
        (*self).rdh3()
    }

    #[inline]
    fn link_id(&self) -> u8 {
        (*self).link_id()
    }

    #[inline]
    fn payload_size(&self) -> u16 {
        (*self).payload_size()
    }

    #[inline]
    fn offset_to_next(&self) -> u16 {
        (*self).offset_to_next()
    }

    #[inline]
    fn stop_bit(&self) -> u8 {
        (*self).stop_bit()
    }

    #[inline]
    fn pages_counter(&self) -> u16 {
        (*self).pages_counter()
    }

    #[inline]
    fn data_format(&self) -> u8 {
        (*self).data_format()
    }

    #[inline]
    fn trigger_type(&self) -> u32 {
        (*self).trigger_type()
    }

    #[inline]
    fn fee_id(&self) -> u16 {
        (*self).fee_id()
    }

    #[inline]
    fn cru_id(&self) -> u16 {
        (*self).cru_id()
    }

    #[inline]
    fn dw(&self) -> u8 {
        (*self).dw()
    }

    #[inline]
    fn packet_counter(&self) -> u8 {
        (*self).packet_counter()
    }
}

/// Trait to Serialize/Deserialise (serde) [RDH] words.
pub trait SerdeRdh: Send + Sync + Sized
where
    Self: ByteSlice,
{
    /// Deserializes a [RDH] from a reader where the next 64 bytes contain a [RDH]
    #[inline]
    fn load<R: std::io::Read>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let buf = macros::load_bytes!(64, reader);
        Self::from_buf(&buf)
    }

    /// Deserializes a [RDH] from a [RDH0][Rdh0] and a reader where the next 56 bytes contain the rest of the [RDH]
    #[inline]
    fn load_from_rdh0<R: std::io::Read>(
        reader: &mut R,
        rdh0: Rdh0,
    ) -> Result<Self, std::io::Error> {
        let buf = macros::load_bytes!(56, reader);
        Self::from_rdh0_and_buf(rdh0, &buf)
    }

    /// Serializes a [RDH] from a byte slice
    #[inline]
    fn from_buf(buf: &[u8]) -> Result<Self, std::io::Error> {
        let rdh0 = Rdh0::from_buf(&buf[0..=7])?;
        Self::from_rdh0_and_buf(rdh0, &buf[8..=63])
    }

    /// Deserializes a [RDH] from a [RDH0][Rdh0] and a byte slice
    fn from_rdh0_and_buf(rdh0: Rdh0, buf: &[u8]) -> Result<Self, std::io::Error>;
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
impl ByteSlice for RdhCru {}

/// # Safety
/// This function can only be used to serialize a struct if it has the #[repr(packed)] attribute
/// If there's any padding on T, it is UNITIALIZED MEMORY and therefor UNDEFINED BEHAVIOR!
#[inline]
unsafe fn any_as_u8_slice<T: Sized>(packed: &T) -> &[u8] {
    use core::{mem::size_of, slice::from_raw_parts};
    // Create read-only reference to T as a byte slice, safe as long as no padding bytes are read
    let ptr_to_packed_data: *const T = packed;
    from_raw_parts(ptr_to_packed_data as *const u8, size_of::<T>())
}

/// Module containing macros related to protocol words.
pub mod macros {
    #[macro_export]
    /// Macro to load a given number of bytes from a reader into a byte array buffer, to avoid heap allocation.
    macro_rules! load_bytes {
        ($size:literal, $reader:ident) => {{
            // Create a buffer array of the given size
            let mut buf = [0u8; $size];
            // Read into the buffer
            $reader.read_exact(&mut buf)?;
            buf
        }};
    }
    pub use load_bytes;
}
