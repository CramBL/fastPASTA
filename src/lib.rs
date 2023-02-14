use std::path::PathBuf;

pub mod data_words;
pub mod macros;
pub mod validators;

pub trait GbtWord: std::fmt::Debug + PartialEq {
    fn print(&self);
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
}

pub trait ByteSlice {
    fn to_byte_slice(&self) -> &[u8];
}

/// # Safety
/// This function can only be used to serialize a struct if it has the #[repr(packed)] attribute
/// If there's any padding on T, it is UNITIALIZED MEMORY and therefor UNDEFINED BEHAVIOR!
#[inline]
pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    // Create read-only reference to T as a byte slice, safe as long as no padding bytes are read
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

#[inline]
pub fn file_open_read_only(path: &PathBuf) -> std::io::Result<std::fs::File> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(path)
        .expect("File not found");
    Ok(file)
}

#[inline(always)]
pub fn buf_reader_with_capacity(
    file: std::fs::File,
    capacity: usize,
) -> std::io::BufReader<std::fs::File> {
    std::io::BufReader::with_capacity(capacity, file)
}
