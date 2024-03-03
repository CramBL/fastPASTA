//! Data definitions for ITS payload words

use std::fmt;
pub mod alpide;
pub mod data_words;
pub mod lane_data_frame;
pub mod status_words;

/// Enum for marking if the data is from the inner/middle/outer layer
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Layer {
    /// Data is from the inner layer
    Inner,
    /// Data is from the middle layer
    Middle,
    /// Data is from the outer layer
    Outer,
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Layer::Inner => write!(f, "IL"),
            Layer::Middle => write!(f, "ML"),
            Layer::Outer => write!(f, "OL"),
        }
    }
}

impl Layer {
    /// Get the [Layer] from a [Stave] (which layer a given stave is from)
    pub fn from_stave(stave: &Stave) -> Self {
        match stave {
            Stave::InnerLayer { .. } => Layer::Inner,
            Stave::MiddleLayer { .. } => Layer::Middle,
            Stave::OuterLayer { .. } => Layer::Outer,
        }
    }
}

/// Enum for marking if the data is from the inner/middle/outer layer along with the stave number
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Stave {
    /// Stave from the inner layer
    InnerLayer {
        /// Layer number
        layer: u8,
        /// Stave number
        stave: u8,
    },
    /// Stave from the middle layer
    MiddleLayer {
        /// Layer number
        layer: u8,
        /// Stave number
        stave: u8,
    },
    /// Stave from the outer layer
    OuterLayer {
        /// Layer number
        layer: u8,
        /// Stave number
        stave: u8,
    },
}

impl fmt::Display for Stave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stave::InnerLayer { layer, stave } => write!(f, "L{layer}_{stave}"),
            Stave::MiddleLayer { layer, stave } => write!(f, "L{layer}_{stave}"),
            Stave::OuterLayer { layer, stave } => write!(f, "L{layer}_{stave}"),
        }
    }
}

impl Stave {
    /// Create a Stave from a FEE ID
    ///
    /// # Example
    /// ```
    /// # use fastpasta::words::its::Stave;
    /// let fee_id: u16 = 524;
    /// let stave = Stave::from_feeid(fee_id);
    /// assert_eq!(stave, Stave::InnerLayer { layer: 0, stave: 12 });
    /// ```
    pub fn from_feeid(fee_id: u16) -> Self {
        let layer = layer_from_feeid(fee_id);
        let stave = stave_number_from_feeid(fee_id);
        match layer {
            0..=2 => Stave::InnerLayer { layer, stave },
            3 | 4 => Stave::MiddleLayer { layer, stave },
            5 | 6 => Stave::OuterLayer { layer, stave },
            _ => panic!("Invalid layer number"),
        }
    }
    /// Get the layer number
    pub fn layer(&self) -> u8 {
        match self {
            Stave::InnerLayer { layer, .. } => *layer,
            Stave::MiddleLayer { layer, .. } => *layer,
            Stave::OuterLayer { layer, .. } => *layer,
        }
    }

    /// Get the stave number
    pub fn stave(&self) -> u8 {
        match self {
            Stave::InnerLayer { stave, .. } => *stave,
            Stave::MiddleLayer { stave, .. } => *stave,
            Stave::OuterLayer { stave, .. } => *stave,
        }
    }
}

// Utility functions to extract information from the FeeId
/// Extracts stave_number from 6 LSB \[5:0\]
///
/// # Example
/// ```
/// # use fastpasta::words::its::stave_number_from_feeid;
/// let fee_id: u16 = 524;
/// let stave_number: u8 = stave_number_from_feeid(fee_id);
/// assert_eq!(stave_number, 12);
/// ```
pub fn stave_number_from_feeid(fee_id: u16) -> u8 {
    let stave_number_mask: u16 = 0b11_1111;
    (fee_id & stave_number_mask) as u8
}

/// Extracts layer number from 3 bits \[14:12\]
pub fn layer_from_feeid(fee_id: u16) -> u8 {
    // Extract layer from 3 bits 14:12
    let layer_mask: u16 = 0b0111;
    let layer_lsb_idx: u8 = 12;
    ((fee_id >> layer_lsb_idx) & layer_mask) as u8
}

/// Convert layer and stave number to fee_id, assumes Link 0.
/// # Example
/// ```
/// # use fastpasta::words::its::feeid_from_layer_stave;
/// let fee_id = feeid_from_layer_stave(0, 12);
/// assert_eq!(fee_id, 12);
/// ```
pub fn feeid_from_layer_stave(layer: u8, stave: u8) -> u16 {
    let layer_mask: u16 = 0b0111;
    let stave_mask: u16 = 0b11_1111;
    let layer_lsb_idx: u8 = 12;
    let stave_lsb_idx: u8 = 0;
    ((layer as u16 & layer_mask) << layer_lsb_idx) | (stave as u16 & stave_mask) << stave_lsb_idx
}

/// Compare the two FEE IDs, ignoring the link ID.
/// # Example
/// ```
/// # use fastpasta::words::its::is_match_feeid_layer_stave;
/// let fee_id_a = 20522; // Link 0
/// let fee_id_b = 20778; // Link 1
/// assert!(is_match_feeid_layer_stave(fee_id_a, fee_id_b));
/// ```
/// ```
/// /// Trivial example
/// # use fastpasta::words::its::is_match_feeid_layer_stave;
/// let fee_id_a = 20522;
/// let fee_id_b = 20522;
/// assert!(is_match_feeid_layer_stave(fee_id_a, fee_id_b));
/// ```
/// ```
/// /// Same layer, different stave
/// # use fastpasta::words::its::is_match_feeid_layer_stave;
/// let fee_id_a = 20522;
/// let fee_id_b = 20523;
/// assert!(!is_match_feeid_layer_stave(fee_id_a, fee_id_b));
/// ```
///
pub fn is_match_feeid_layer_stave(a_fee_id: u16, b_fee_id: u16) -> bool {
    let layer_stave_mask: u16 = 0b0111_0000_0011_1111;
    (a_fee_id & layer_stave_mask) == (b_fee_id & layer_stave_mask)
}

/// Convert a string of the form "layer_stave" to a FEE ID, assumes Link 0.
///
/// # Examples
/// ```
/// /// "L5_42" -> 20522
/// # use fastpasta::words::its::layer_stave_string_to_feeid;
/// let fee_id = layer_stave_string_to_feeid(&String::from("L5_42"));
/// assert_eq!(fee_id, Some(20522));
/// ```
/// ```
/// /// "l0_1" -> 1
/// # use fastpasta::words::its::layer_stave_string_to_feeid;
/// let fee_id = layer_stave_string_to_feeid(&String::from("l0_1"));
/// assert_eq!(fee_id, Some(1));
/// ```
pub fn layer_stave_string_to_feeid(layer_stave_str: &str) -> Option<u16> {
    let split_fee = layer_stave_str.split('_').collect::<Vec<&str>>();
    debug_assert!(split_fee.len() == 2);
    // 5. Parse the first string that should contain the layer number
    if let Ok(layer_num) = split_fee[0].get(1..)?.parse::<u8>() {
        // 6. Parse the second string that should contain the stave number
        if let Ok(stave_num) = split_fee[1].parse::<u8>() {
            // 7. Return the FEE ID
            Some(feeid_from_layer_stave(layer_num, stave_num))
        } else {
            None
        }
    } else {
        None
    }
}

/// Payload test values for ITS
pub mod test_payloads {
    /// The beginning of a payload in flavor 0
    pub const START_PAYLOAD_FLAVOR_0: [u8; 32] = [
        0xC0, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x03, 0x1a, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0xE8, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    /// The beginning of a payload in flavor 2
    pub const START_PAYLOAD_FLAVOR_2: [u8; 20] = [
        0x38, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0x13, 0x08, 0x00, 0x00, 0x00,
        0xD7, 0x39, 0x9B, 0x00, 0xE8,
    ];

    /// Middle of a payload in flavor 0, just one Data Word with padding
    pub const MIDDLE_PAYLOAD_FLAVOR_0: [u8; 16] = [
        0xA7, 0x00, 0xC0, 0x41, 0xFF, 0xB0, 0x00, 0x00, 0x00, 0x27, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ];

    /// Middle of a payload in flavor 2, two packed Data Words
    pub const MIDDLE_PAYLOAD_FLAVOR_2: [u8; 20] = [
        0xA7, 0x00, 0xC0, 0x41, 0xFF, 0xB0, 0x00, 0x00, 0x00, 0x27, 0xA8, 0x00, 0xC0, 0x41, 0xFF,
        0xB0, 0x00, 0x00, 0x00, 0x28,
    ];

    /// End of a payload in flavor 0: has no 0xFF padding, this is just a TDT followed by the 0x00 padding
    pub const END_PAYLOAD_FLAVOR_0: [u8; 16] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];
    /// End of a payload in flavor 2: TDT and 0xFF padding
    pub const END_PAYLOAD_FLAVOR_2: [u8; 16] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_stave_feeid_coversions() {
        let layer = 2;
        let stave = 0;
        println!("Layer/stave L{layer}_{stave}");
        let feeid = feeid_from_layer_stave(layer, stave);
        println!("feeid: {feeid}");
        assert_eq!(layer, layer_from_feeid(feeid));
        assert_eq!(stave, stave_number_from_feeid(feeid));
    }

    #[test]
    fn feeid_layer_stave_conversion() {
        let feeid = 20522;
        let layer = layer_from_feeid(feeid);
        let stave = stave_number_from_feeid(feeid);
        println!("Layer/stave L{layer}_{stave}");
        assert_eq!(feeid, feeid_from_layer_stave(layer, stave));
    }

    #[test]
    fn test_layer_stave_string_to_feeid() {
        let fee_id = layer_stave_string_to_feeid(&String::from("L5_42"));
        println!("feeid: {:?}", fee_id);
        assert_eq!(fee_id, Some(20522));
    }
}
