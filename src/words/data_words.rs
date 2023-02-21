pub trait StatusWord: std::fmt::Debug + PartialEq + Sized + ByteSlice {
    fn id(&self) -> u8;
    fn print(&self);
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn is_reserved_0(&self) -> bool;
}

#[repr(packed)]
pub struct ItsDataWord {
    pub dw0: u8,
    pub dw1: u8,
    pub dw2: u8,
    pub dw3: u8,
    pub dw4: u8,
    pub dw5: u8,
    pub dw6: u8,
    pub dw7: u8,
    pub dw8: u8,
    pub id: u8,
}

const VALID_OL_IDS: [u8; 8] = [0x40, 0x46, 0x48, 0x4E, 0x50, 0x56, 0x58, 0x5E];
const VALID_ML_IDS: [u8; 8] = [0x43, 0x46, 0x48, 0x4B, 0x53, 0x56, 0x58, 0x5B];
const UNION_VALID_IDS: [u8; 12] = [
    0x40, 0x43, 0x46, 0x48, 0x4B, 0x4E, 0x50, 0x53, 0x56, 0x58, 0x5B, 0x5E,
];
