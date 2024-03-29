//! Definitions for status words: [IHW][Ihw], [TDH][Tdh], [TDT][Tdt], [DDW0][Ddw0] & [CDW][Cdw].

use crate::util::*;
use alice_protocol_reader::prelude::macros::load_bytes;

pub mod cdw;
pub mod ddw;
pub mod ihw;
pub mod tdh;
pub mod tdt;

pub mod util;

impl alice_protocol_reader::prelude::ByteSlice for Ihw {}
impl alice_protocol_reader::prelude::ByteSlice for Tdh {}
impl alice_protocol_reader::prelude::ByteSlice for Cdw {}
impl alice_protocol_reader::prelude::ByteSlice for Tdt {}
impl alice_protocol_reader::prelude::ByteSlice for Ddw0 {}

/// Trait to implement for all status words
pub trait StatusWord:
    fmt::Debug + PartialEq + Sized + alice_protocol_reader::prelude::ByteSlice + fmt::Display
{
    /// Returns the id of the status word
    fn id(&self) -> u8;
    /// Deserializes the status word from a reader and a byte slice
    #[inline]
    fn load<T: io::Read>(reader: &mut T) -> Result<Self, io::Error>
    where
        Self: Sized,
    {
        let buf = load_bytes!(10, reader);
        Self::from_buf(&buf)
    }
    /// Deserializes the GBT word from a byte slice
    fn from_buf(buf: &[u8]) -> Result<Self, io::Error>;
    /// Sanity check that returns true if all reserved bits are 0
    fn is_reserved_0(&self) -> bool;
}

/// Helper to display all the status words in a similar way, without dynamic dispatch
#[inline]
fn display_byte_slice<T: StatusWord>(status_word: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let slice = status_word.to_byte_slice();
    for (i, byte) in slice.iter().enumerate() {
        if i > 0 {
            // Add a space before every byte after the first
            f.write_str(" ")?;
        }
        // Use format_args! to defer formatting, avoiding intermediate strings
        write!(f, "{}", format_args!("{:02X}", byte))?;
    }
    Ok(())
}
