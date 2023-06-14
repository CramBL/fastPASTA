//! Contains utility functions for working with data words in a CDP payload

use std::ops::RangeInclusive;

/// Takes an OL/ML (OB) data word ID and returns the lane number
#[inline]
pub fn ob_data_word_id_to_lane(data_word_id: u8) -> u8 {
    // let lane_id = data_word_id & 0x1F;
    if data_word_id <= *VALID_OL_CONNECT0_ID.end() {
        // 0-6
        data_word_id % VALID_OL_CONNECT0_ID.start()
    } else if data_word_id <= *VALID_OL_CONNECT1_ID.end() {
        // 7-13
        7 + (data_word_id % VALID_OL_CONNECT1_ID.start())
    } else if data_word_id <= *VALID_OL_CONNECT2_ID.end() {
        // 14-20
        14 + (data_word_id % VALID_OL_CONNECT2_ID.start())
    } else {
        // 21-27
        21 + (data_word_id % VALID_OL_CONNECT3_ID.start())
    }
}

/// Takes an ob data word ID and returns the input connector number
#[inline]
pub fn ob_data_word_id_to_input_number_connector(data_word_id: u8) -> u8 {
    // [2:0] = input number on the connector
    //     * 0b000 - 0b110 (0-6 on the connector)
    data_word_id & 0b111
}

/// Takes an ob data word ID and returns the connector number
#[inline]
pub fn ob_data_word_id_to_connector(data_word_id: u8) -> u8 {
    // [4:3] = connector number
    //     * 0b00 - 0b11 (0-3)
    (data_word_id >> 3) & 0b11
}

/// Takes an IL/IB data word ID and returns the lane number
#[inline]
pub fn ib_data_word_id_to_lane(data_word_id: u8) -> u8 {
    // let lane_id = data_word_id & 0x1F;
    data_word_id & 0x1F
}

// IDs are defined as follows:
// 7:5 =
//   * 0b000 = reserved
//   * 0b001 = Inner Barrel Data (layers 0,1,2)
//   * 0b010 = Outer Barrel Data (layers 3,4,5,6)
//   * 0b111 = Status Word
// 4:0 for INNER BARREL = lane number
//   * 0b00000 - 0b1000 (0-8)
// 4:0 for OUTER BARREL =
//   divided into 2 groups of [4:3] and [2:0] =
//      [4:3] = connector number
//           * 0b00 - 0b11 (equivelant to 1/4th stave)
//      [2:0] = input number on the connector
//           * 0b000 - 0b111 (0-6 on the connector)

/// Convenience tuple of the min/max range for the ID of an IL data word (9 lanes)
pub const VALID_IL_ID: RangeInclusive<u8> = 0x20..=0x28;

// Tuples for ID ranges of the 16 ML lanes
/// Convenience tuple of the min/max range for the ID of an ML data word from connector 0
pub const VALID_ML_CONNECT0_ID: RangeInclusive<u8> = 0x43..=0x46;
/// Convenience tuple of the min/max range for the ID of an ML data word from connector 1
pub const VALID_ML_CONNECT1_ID: RangeInclusive<u8> = 0x48..=0x4B;
/// Convenience tuple of the min/max range for the ID of an ML data word from connector 2
pub const VALID_ML_CONNECT2_ID: RangeInclusive<u8> = 0x53..=0x56;
/// Convenience tuple of the min/max range for the ID of an ML data word from connector 3
pub const VALID_ML_CONNECT3_ID: RangeInclusive<u8> = 0x58..=0x5B;

// Tuples for the ID ranges of the 28 OL lanes
/// Convenience tuple of the min/max range for the ID of an OL data word from connector 0
pub const VALID_OL_CONNECT0_ID: RangeInclusive<u8> = 0x40..=0x46;
/// Convenience tuple of the min/max range for the ID of an OL data word from connector 1
pub const VALID_OL_CONNECT1_ID: RangeInclusive<u8> = 0x48..=0x4E;
/// Convenience tuple of the min/max range for the ID of an OL data word from connector 2
pub const VALID_OL_CONNECT2_ID: RangeInclusive<u8> = 0x50..=0x56;
/// Convenience tuple of the min/max range for the ID of an OL data word from connector 3
pub const VALID_OL_CONNECT3_ID: RangeInclusive<u8> = 0x58..=0x5E;

/// Newtype for the inner barrel, to avoid comparing lanes from different barrels (zero cost abstraction)
#[repr(transparent)]
#[derive(PartialEq, PartialOrd)]
pub struct IbLane(u8);
/// Newtype for the outer barrel, to avoid comparing lanes from different barrels (zero cost abstraction)
#[repr(transparent)]
#[derive(PartialEq, PartialOrd)]
pub struct ObLane(u8);

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const DATA_WORDS_OB: [u8; 64] = [
        0xA8, 0x00, 0xC0, 0x01, 0xFE, 0x7F, 0x05, 0xFE, 0x7F, 0x46, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xA0, 0x00, 0xC0, 0x01, 0xFE, 0x7F, 0x05, 0xFE, 0x7F, 0x48, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xA0, 0x00, 0xC0, 0x01, 0xFE, 0x7F, 0x05, 0xFE, 0x7F, 0x49, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xA0, 0x00, 0xC0, 0x01, 0xFE, 0x7F, 0x05, 0xFE, 0x7F, 0x4A, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ]; // 4 data words with IDs:     46, 48, 49, 4A:
       //   - lane:                   6,  7,  8,  9
       //   - input number connector: 6,  0,  1,  2
       //   - connector number:       0,  1,  1,  1

    #[test]
    fn test_ob_data_word_id_to_lane() {
        let data_words_ob = DATA_WORDS_OB.chunks_exact(16); // data format 0 has 6 bytes of padding

        data_words_ob
            .into_iter()
            .enumerate()
            .for_each(|(idx, data_word)| {
                let id = data_word[9];
                let lane = ob_data_word_id_to_lane(id);
                assert_eq!(lane, (idx + 6) as u8);
            });
    }

    #[test]
    fn test_ob_data_word_id_to_input_number_connector() {
        let data_words_ob = DATA_WORDS_OB.chunks_exact(16); // data format 0 has 6 bytes of padding
        let correct_input_number_connector = [6, 0, 1, 2];
        data_words_ob
            .into_iter()
            .enumerate()
            .for_each(|(idx, data_word)| {
                let id = data_word[9];
                let input_number_connector = ob_data_word_id_to_input_number_connector(id);
                assert_eq!(input_number_connector, correct_input_number_connector[idx])
            });
    }

    #[test]
    fn test_ob_data_word_id_to_connector() {
        let data_words_ob = DATA_WORDS_OB.chunks_exact(16); // data format 0 has 6 bytes of padding
        let correct_connector = [0, 1, 1, 1];
        data_words_ob
            .into_iter()
            .enumerate()
            .for_each(|(idx, data_word)| {
                let id = data_word[9];
                let connector = ob_data_word_id_to_connector(id);
                assert_eq!(connector, correct_connector[idx])
            });
    }
}
