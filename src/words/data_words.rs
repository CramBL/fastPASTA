#![allow(dead_code)]
use std::fmt::Display;

use crate::ByteSlice;

pub trait DataWord: std::fmt::Display + PartialEq + Sized + ByteSlice {
    fn lane(&self) -> u8;
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
}

// Helper to display all the data words in a similar way, without dynamic dispatch
#[inline]
fn display_byte_slice<T: DataWord>(
    data_word: &T,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let slice = data_word.to_byte_slice();
    write!(
        f,
        "{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} [79:0]",
        slice[9],
        slice[8],
        slice[7],
        slice[6],
        slice[5],
        slice[4],
        slice[3],
        slice[2],
        slice[1],
        slice[0],
    )
}

// IDs are defined as follows:
// 7:5 =
//   * 0b0   = reserved
//   * 0b1   = Inner Barrel Data (layers 0,1,2)
//   * 0b10  = Outer Barrel Data (layers 3,4,5,6)
//   * 0b111 = Status Word
// 4:0 for INNER BARREL = lane number
//   * 0b00000 - 0b1000 (0-8)
// 4:0 for OUTER BARREL =
//   divided into 2 groups of [4:3] and [2:0] =
//      [4:3] = connector number
//           * 0b00 - 0b11 (equivelant to 1/4th stave)
//      [2:0] = input number on the connector
//           * 0b000 - 0b111 (0-6 on the connector)
// 9 lanes
pub const VALID_IL_ID_MIN_MAX: (u8, u8) = (0x20, 0x28);

// 16 lanes
pub const VALID_ML_CONNECT0_ID_MIN_MAX: (u8, u8) = (0x43, 0x46);
pub const VALID_ML_CONNECT1_ID_MIN_MAX: (u8, u8) = (0x48, 0x4B);
pub const VALID_ML_CONNECT2_ID_MIN_MAX: (u8, u8) = (0x53, 0x56);
pub const VALID_ML_CONNECT3_ID_MIN_MAX: (u8, u8) = (0x58, 0x5B);

// 28 lanes
pub const VALID_OL_CONNECT0_ID_MIN_MAX: (u8, u8) = (0x40, 0x46);
pub const VALID_OL_CONNECT1_ID_MIN_MAX: (u8, u8) = (0x48, 0x4E);
pub const VALID_OL_CONNECT2_ID_MIN_MAX: (u8, u8) = (0x50, 0x56);
pub const VALID_OL_CONNECT3_ID_MIN_MAX: (u8, u8) = (0x58, 0x5E);

// Newtypes for the inner/outer barrel, to avoid comparing lanes from different barrels, with 0 runtime cost

#[repr(packed)]
#[derive(PartialEq, PartialOrd)]
pub struct ItsDataWordIb {
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

impl DataWord for ItsDataWordIb {
    fn lane(&self) -> u8 {
        self.id & 0x1F
    }
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut data = [0u8; 10];
        reader.read_exact(&mut data)?;
        Ok(Self {
            dw0: data[0],
            dw1: data[1],
            dw2: data[2],
            dw3: data[3],
            dw4: data[4],
            dw5: data[5],
            dw6: data[6],
            dw7: data[7],
            dw8: data[8],
            id: data[9],
        })
    }
}

impl ByteSlice for ItsDataWordIb {
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { crate::any_as_u8_slice(self) }
    }
}

impl Display for ItsDataWordIb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_byte_slice(self, f)
    }
}

#[repr(packed)]
#[derive(PartialEq, PartialOrd)]
pub struct ItsDataWordOb {
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

impl Display for ItsDataWordOb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_byte_slice(self, f)
    }
}

impl ByteSlice for ItsDataWordOb {
    fn to_byte_slice(&self) -> &[u8] {
        unsafe { crate::any_as_u8_slice(self) }
    }
}

impl DataWord for ItsDataWordOb {
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut data = [0u8; 10];
        reader.read_exact(&mut data)?;
        Ok(Self {
            dw0: data[0],
            dw1: data[1],
            dw2: data[2],
            dw3: data[3],
            dw4: data[4],
            dw5: data[5],
            dw6: data[6],
            dw7: data[7],
            dw8: data[8],
            id: data[9],
        })
    }
    fn lane(&self) -> u8 {
        let lane_id = self.id & 0x1F;
        ob_lane(ObLane(lane_id))
    }
}

fn ob_lane(ob_id: ObLane) -> u8 {
    let lane_id = ob_id.0;
    if lane_id < VALID_OL_CONNECT0_ID_MIN_MAX.1 {
        // 0-6
        lane_id % VALID_OL_CONNECT0_ID_MIN_MAX.0
    } else if lane_id < VALID_OL_CONNECT1_ID_MIN_MAX.1 {
        // 7-13
        7 + (lane_id % VALID_OL_CONNECT1_ID_MIN_MAX.0)
    } else if lane_id < VALID_OL_CONNECT2_ID_MIN_MAX.1 {
        // 14-20
        14 + (lane_id % VALID_OL_CONNECT2_ID_MIN_MAX.0)
    } else {
        // 21-27
        21 + (lane_id % VALID_OL_CONNECT3_ID_MIN_MAX.0)
    }
}

// Newtypes for the inner/outer barrel, to avoid comparing lanes from different barrels, with 0 runtime cost
#[repr(transparent)]
#[derive(PartialEq, PartialOrd)]
pub struct IbLane(u8);
#[repr(transparent)]
#[derive(PartialEq, PartialOrd)]
pub struct ObLane(u8);

/// Takes in an IB lane number and a byte slice (data word), returns an ItsDataWordIb if the lane number matches the data word
pub fn data_word_lane_filter_ib(ib_lane: IbLane, data_word: &[u8]) -> Option<ItsDataWordIb> {
    let lane_id = data_word[9] & 0x1F;
    if ib_lane.0 == lane_id {
        #[allow(clippy::useless_asref)] // Actual false negative
        let data_word = ItsDataWordIb::load(&mut data_word.as_ref()).unwrap();
        Some(data_word)
    } else {
        None
    }
}

/// Takes in an OB lane number and a byte slice (data word), returns an ItsDataWordOb if the lane number matches the data word
pub fn data_word_lane_filter_ob(ob_lane_num: ObLane, data_word: &[u8]) -> Option<ItsDataWordOb> {
    let lane_id = data_word[9] & 0x1F;
    let lane = ob_lane(ObLane(lane_id));
    if ob_lane_num.0 == lane {
        #[allow(clippy::useless_asref)] // Actual false negative
        let data_word = ItsDataWordOb::load(&mut data_word.as_ref()).unwrap();
        Some(data_word)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA_WORDS_OB: [u8; 64] = [
        0xA8, 0x00, 0xC0, 0x01, 0xFE, 0x7F, 0x05, 0xFE, 0x7F, 0x46, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xA0, 0x00, 0xC0, 0x01, 0xFE, 0x7F, 0x05, 0xFE, 0x7F, 0x48, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xA0, 0x00, 0xC0, 0x01, 0xFE, 0x7F, 0x05, 0xFE, 0x7F, 0x49, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xA0, 0x00, 0xC0, 0x01, 0xFE, 0x7F, 0x05, 0xFE, 0x7F, 0x4A, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn test_filter_ob_lane_6_found() {
        let lane = 6;
        let ob_id_lane6 = 0x46;
        let data_words: [u8; 64] = DATA_WORDS_OB;
        let chunks = data_words.chunks_exact(16).collect::<Vec<_>>();

        let first_data_word = chunks.first().unwrap();
        assert_eq!(first_data_word[9], ob_id_lane6);
        let data_word = data_word_lane_filter_ob(ObLane(lane), chunks.first().unwrap());
        assert!(data_word.is_some());
        match data_word {
            Some(data_word) => {
                println!("Data word found: {}", data_word);
            }
            None => {
                println!("No data word found");
            }
        }
    }

    #[test]
    fn test_filter_ib_lane_2_not_found() {
        let lane = 6;

        let ob_id_lane6 = 0x46;
        let data_words: [u8; 64] = DATA_WORDS_OB;
        let chunks = data_words.chunks_exact(16).collect::<Vec<_>>();

        let first_data_word = chunks.first().unwrap();
        assert_eq!(first_data_word[9], ob_id_lane6);
        let data_word = data_word_lane_filter_ib(IbLane(lane), chunks.first().unwrap());
        assert!(data_word.is_some());
        match data_word {
            Some(data_word) => {
                println!("OB Lane {lane} Data word found: {}", data_word);
            }
            None => {
                println!("No data word found");
            }
        }
    }

    #[test]
    fn test_valid_valids() {
        let (min, max) = VALID_IL_ID_MIN_MAX;
        assert!(min <= max);
        let (min, max) = VALID_ML_CONNECT0_ID_MIN_MAX;
        assert!(min <= max);
        let (min, max) = VALID_ML_CONNECT1_ID_MIN_MAX;
        assert!(min <= max);
        let (min, max) = VALID_ML_CONNECT2_ID_MIN_MAX;
        assert!(min <= max);
        let (min, max) = VALID_ML_CONNECT3_ID_MIN_MAX;
        assert!(min <= max);
        let (min, max) = VALID_OL_CONNECT0_ID_MIN_MAX;
        assert!(min <= max);
        let (min, max) = VALID_OL_CONNECT1_ID_MIN_MAX;
        assert!(min <= max);
        let (min, max) = VALID_OL_CONNECT2_ID_MIN_MAX;
        assert!(min <= max);
        let (min, max) = VALID_OL_CONNECT3_ID_MIN_MAX;
        assert!(min <= max);
    }

    #[test]
    fn test_valid_il() {
        let (min, max) = VALID_IL_ID_MIN_MAX;
        for i in min..=max {
            let dw = ItsDataWordIb {
                dw0: 0,
                dw1: 0,
                dw2: 0,
                dw3: 0,
                dw4: 0,
                dw5: 0,
                dw6: 0,
                dw7: 0,
                dw8: 0,
                id: i,
            };
            assert!(dw.lane() < 9);
        }
    }

    #[test]
    fn invalid_compare() {
        let (min, max) = VALID_IL_ID_MIN_MAX;
        for i in min..=max {
            let dw = ItsDataWordIb {
                dw0: 0,
                dw1: 0,
                dw2: 0,
                dw3: 0,
                dw4: 0,
                dw5: 0,
                dw6: 0,
                dw7: 0,
                dw8: 0,
                id: i,
            };
            let ib_lane = IbLane(i & 0x1F);
            let ib_lane_get = dw.lane();
            assert!(ib_lane == ib_lane);
            assert!(ib_lane.0 == ib_lane_get);
            assert!(ib_lane == IbLane(dw.lane()));
        }
    }

    #[test]
    fn valid_ob_0() {
        let (min, max) = VALID_OL_CONNECT0_ID_MIN_MAX;
        for i in min..=max {
            let dw = ItsDataWordOb {
                dw0: 0,
                dw1: 0,
                dw2: 0,
                dw3: 0,
                dw4: 0,
                dw5: 0,
                dw6: 0,
                dw7: 0,
                dw8: 0,
                id: i,
            };
            assert!(dw.lane() < 7);
        }
    }

    #[test]
    fn valid_ob_1() {
        let (min, max) = VALID_OL_CONNECT1_ID_MIN_MAX;
        for i in min..=max {
            let dw = ItsDataWordOb {
                dw0: 0,
                dw1: 0,
                dw2: 0,
                dw3: 0,
                dw4: 0,
                dw5: 0,
                dw6: 0,
                dw7: 0,
                dw8: 0,
                id: i,
            };
            println!("{}", ObLane(dw.lane()).0);
            assert!(dw.lane() > 7);
            assert!(dw.lane() < 15);
        }
    }

    #[test]
    fn valid_ob_2() {
        let (min, max) = VALID_OL_CONNECT2_ID_MIN_MAX;
        for i in min..=max {
            let dw = ItsDataWordOb {
                dw0: 0,
                dw1: 0,
                dw2: 0,
                dw3: 0,
                dw4: 0,
                dw5: 0,
                dw6: 0,
                dw7: 0,
                dw8: 0,
                id: i,
            };
            println!("{}", dw.lane());
            assert!(dw.lane() > 15);
            assert!(dw.lane() < 23);
        }
    }

    #[test]
    fn valid_ob_3() {
        let (min, max) = VALID_OL_CONNECT3_ID_MIN_MAX;
        for i in min..=max {
            let dw = ItsDataWordOb {
                dw0: 0,
                dw1: 0,
                dw2: 0,
                dw3: 0,
                dw4: 0,
                dw5: 0,
                dw6: 0,
                dw7: 0,
                dw8: 0,
                id: i,
            };
            println!("{}", dw.lane());
            assert!(dw.lane() > 23);
            assert!(dw.lane() < 31);
        }
    }
}
