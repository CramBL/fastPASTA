#![allow(dead_code)]
use crate::ByteSlice;

pub trait StatusWord: std::fmt::Debug + PartialEq + Sized + ByteSlice {
    fn id(&self) -> u8;
    fn lane(&self) -> u8;
    fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn is_reserved_0(&self) -> bool;
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
const VALID_IL_ID_MIN_MAX: (u8, u8) = (0x20, 0x28);

// 16 lanes
const VALID_ML_CONNECT0_ID_MIN_MAX: (u8, u8) = (0x43, 0x46);
const VALID_ML_CONNECT1_ID_MIN_MAX: (u8, u8) = (0x48, 0x4B);
const VALID_ML_CONNECT2_ID_MIN_MAX: (u8, u8) = (0x53, 0x56);
const VALID_ML_CONNECT3_ID_MIN_MAX: (u8, u8) = (0x58, 0x5B);

// 28 lanes
const VALID_OL_CONNECT0_ID_MIN_MAX: (u8, u8) = (0x40, 0x46);
const VALID_OL_CONNECT1_ID_MIN_MAX: (u8, u8) = (0x48, 0x4E);
const VALID_OL_CONNECT2_ID_MIN_MAX: (u8, u8) = (0x50, 0x56);
const VALID_OL_CONNECT3_ID_MIN_MAX: (u8, u8) = (0x58, 0x5E);

// Newtypes for the inner/outer barrel, to avoid comparing lanes from different barrels, with 0 runtime cost
#[repr(transparent)]
#[derive(PartialEq, PartialOrd)]
pub struct IbLane(u8);

#[repr(transparent)]
#[derive(PartialEq, PartialOrd)]
pub struct ObLane(u8);
#[repr(packed)]
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

impl ItsDataWordIb {
    pub fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
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
    pub fn lane(&self) -> IbLane {
        IbLane(self.id & 0x1F)
    }
}

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

impl ItsDataWordOb {
    pub fn load<T: std::io::Read>(reader: &mut T) -> Result<Self, std::io::Error>
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
    pub fn lane(&self) -> ObLane {
        let lane_id = self.id & 0x1F;
        if lane_id < VALID_OL_CONNECT0_ID_MIN_MAX.1 {
            // 0-6
            ObLane(lane_id % VALID_OL_CONNECT0_ID_MIN_MAX.0)
        } else if lane_id < VALID_OL_CONNECT1_ID_MIN_MAX.1 {
            // 7-13
            ObLane(7 + (lane_id % VALID_OL_CONNECT1_ID_MIN_MAX.0))
        } else if lane_id < VALID_OL_CONNECT2_ID_MIN_MAX.1 {
            // 14-20
            ObLane(14 + (lane_id % VALID_OL_CONNECT2_ID_MIN_MAX.0))
        } else {
            // 21-27
            ObLane(21 + (lane_id % VALID_OL_CONNECT3_ID_MIN_MAX.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            assert!(dw.lane().0 < 9);
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
            assert!(ib_lane.0 == ib_lane_get.0);
            assert!(ib_lane == dw.lane());
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
            assert!(dw.lane() < ObLane(7));
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
            println!("{}", dw.lane().0);
            assert!(dw.lane() > ObLane(7));
            assert!(dw.lane() < ObLane(15));
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
            println!("{}", dw.lane().0);
            assert!(dw.lane() > ObLane(15));
            assert!(dw.lane() < ObLane(23));
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
            println!("{}", dw.lane().0);
            assert!(dw.lane() > ObLane(23));
            assert!(dw.lane() < ObLane(31));
        }
    }
}
