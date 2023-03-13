use super::rdh::{Rdh0, Rdh2};

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
    PartialEq + Sized + crate::ByteSlice + std::fmt::Display + std::fmt::Debug + Sync + Send
{
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn load_from_rdh0<T: std::io::Read>(reader: &mut T, rdh0: Rdh0)
        -> Result<Self, std::io::Error>;
    fn version(&self) -> u8;
    fn rdh2(&self) -> &Rdh2;
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
}
