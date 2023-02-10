pub mod data_words;
pub mod macros;
pub mod validators;

pub trait GbtWord: std::fmt::Debug + PartialEq {
    fn print(&self);
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
}

/// WARNING: if T is a struct you HAVE TO implement repr(packed) because PADDING is UNITIALIZED MEMORY -> UNDEFINED BEHAVIOR!
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    // Create read-only reference to T as a byte slice, safe as long as no padding bytes are read
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}
