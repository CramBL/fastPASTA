//! Definitions for status words: [IHW][Ihw], [TDH][Tdh], [TDT][Tdt], [DDW0][Ddw0] & [CDW][Cdw].

use crate::util::*;
use alice_protocol_reader::prelude::macros::load_bytes;
use cdw::Cdw;
use ddw::Ddw0;
use ihw::Ihw;
use std::fmt;
use tdh::Tdh;
use tdt::Tdt;

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
    write!(
        f,
        "{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
        slice[0],
        slice[1],
        slice[2],
        slice[3],
        slice[4],
        slice[5],
        slice[6],
        slice[7],
        slice[8],
        slice[9],
    )
}
