//! Struct definition of the custom checks configuration TOML file.
use descriptive_toml_derive::TomlConfig;
use serde_derive::{Deserialize, Serialize};

/// Struct for the configuration of various expected counters in the data.
#[derive(TomlConfig, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomChecks {
    #[description = "Number of CRU Data Packets expected in the data"]
    #[example = "20, 500532"]
    cdps: Option<u32>,

    #[description = "Number of Physics (PhT) Triggers expected in the data"]
    #[example = "0, 10"]
    triggers_pht: Option<u32>,

    #[description = "Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of two lists of 7 chip IDs"]
    #[example = "[[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]"]
    chip_orders_ob: Option<(Vec<u8>, Vec<u8>)>,

    #[description = "Number of chips expected in the data from Outer Barrel (ML/OL)"]
    #[example = "7"]
    chip_count_ob: Option<u8>,

    // RDH format specification
    #[description = "The RDH version expected in the data"]
    #[example = "7"]
    rdh_version: Option<u8>,
}

impl CustomChecks {
    /// Get the number of CDPS expected in the data, if it is set.
    pub fn cdps(&self) -> Option<u32> {
        self.cdps
    }

    /// Get the number of sent Triggers expected in the data, if it is set.
    pub fn triggers_pht(&self) -> Option<u32> {
        self.triggers_pht
    }

    /// Get the number of chips expected in the data from Outer Barrel (ML/OL), if it is set.
    pub fn chip_count_ob(&self) -> Option<u8> {
        self.chip_count_ob
    }

    /// Get the expected RDH version, if it is set.
    pub fn rdh_version(&self) -> Option<u8> {
        self.rdh_version
    }

    /// Get the chip orders expected in the data, if it is set.
    ///
    /// Returns a tuple of two slices, representing the legal chip orders for the Outer Barrel (ML/OL).
    pub fn chip_orders_ob(&self) -> Option<(&[u8], &[u8])> {
        self.chip_orders_ob
            .as_ref()
            .map(|(a, b)| (a.as_slice(), b.as_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use temp_dir::TempDir;

    #[test]
    fn test_serde_consistency() {
        let custom_checks = CustomChecks {
            cdps: Some(10),
            triggers_pht: Some(0),
            chip_orders_ob: Some((vec![0, 1, 2, 3, 4, 5, 6], vec![8, 9, 10, 11, 12, 13, 14])),
            chip_count_ob: Some(7),
            rdh_version: Some(7),
        };

        let toml = custom_checks.to_string_pretty_toml();

        let custom_checks_deserde: CustomChecks =
            toml::from_str(&toml).expect("Failed to parse TOML");

        assert_eq!(custom_checks_deserde, custom_checks);
    }

    #[test]
    fn test_serialize_custom_checks() {
        let custom_checks = CustomChecks::default();
        let custom_checks_toml = custom_checks.to_string_pretty_toml();
        println!(
            "==== TOML ====\n{}\n==== End of TOML ====",
            custom_checks_toml
        );
        assert_eq!(
            custom_checks_toml,
            r#"# Number of CRU Data Packets expected in the data
# Example: 20, 500532
#cdps = None [ u32 ] # (Uncomment and set to enable)

# Number of Physics (PhT) Triggers expected in the data
# Example: 0, 10
#triggers_pht = None [ u32 ] # (Uncomment and set to enable)

# Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of two lists of 7 chip IDs
# Example: [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]
#chip_orders_ob = None [ (Vec < u8 >, Vec < u8 >) ] # (Uncomment and set to enable)

# Number of chips expected in the data from Outer Barrel (ML/OL)
# Example: 7
#chip_count_ob = None [ u8 ] # (Uncomment and set to enable)

# The RDH version expected in the data
# Example: 7
#rdh_version = None [ u8 ] # (Uncomment and set to enable)

"#
        );
    }

    #[test]
    fn test_deserialize_counters_default() {
        let custom_checks_toml = r#"
# Number of CRU Data Packets expected in the data
# Example: 20, 500532
#cdps = None [ u32 ] # (Uncomment and set to enable)

# Number of Physics (PhT) Triggers expected in the data
# Example: 0, 10
#triggers_pht = None [ u32 ] # (Uncomment and set to enable)

# Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of two lists of 7 chip IDs
# Example: [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]
#chip_orders_ob = None [ (Vec < u8 >, Vec < u8 >) ] # (Uncomment and set to enable)

# The RDH version expected in the data
# Example: 7
#rdh_version = None [ u8 ] # (Uncomment and set to enable)
"#;
        let custom_checks: CustomChecks = toml::from_str(custom_checks_toml).unwrap();
        assert_eq!(custom_checks, CustomChecks::default());
    }

    #[test]
    fn test_deserialize_counters_some_values() {
        let counters_toml = r#"
# Number of CRU Data Packets expected in the data
# Example: 20, 500532
cdps = 10

# Number of Physics (PhT) Triggers expected in the data
# Example: 0, 10
triggers_pht = 0

# Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of two lists of 7 chip IDs
# Example: [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]
chip_orders_ob = [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]

chip_count_ob = 7

rdh_version = 6

"#;
        let custom_checks: CustomChecks = toml::from_str(counters_toml).unwrap();
        println!("custom checks: {:?}", custom_checks);
        assert_eq!(
            custom_checks,
            CustomChecks {
                cdps: Some(10),
                triggers_pht: Some(0),
                chip_orders_ob: Some((vec![0, 1, 2, 3, 4, 5, 6], vec![8, 9, 10, 11, 12, 13, 14])),
                chip_count_ob: Some(7),
                rdh_version: Some(6)
            }
        );
    }

    #[test]
    fn test_write_toml_to_file() {
        let counters = CustomChecks::default();
        let toml = counters.to_string_pretty_toml();

        let tmp_dir = TempDir::new().unwrap();
        let toml_file = tmp_dir.child("counters.toml");
        std::fs::write(&toml_file, &toml).unwrap();

        assert_eq!(std::fs::read_to_string(toml_file).unwrap(), toml);
    }

    #[test]
    fn test_read_toml_from_file() {
        let counters = CustomChecks::default();
        let toml = counters.to_string_pretty_toml();
        println!("toml string:\n{toml}");
        let tmp_dir = TempDir::new().unwrap();
        let toml_file = tmp_dir.child("counters.toml");
        std::fs::write(&toml_file, &toml).unwrap();
        let toml = std::fs::read_to_string(toml_file).expect("Failed to read TOML file");
        let custom_checks_from_toml: CustomChecks =
            toml::from_str(&toml).expect("Failed to parse TOML");

        println!("Struct from toml file: {:?}", custom_checks_from_toml);
        assert_eq!(custom_checks_from_toml.cdps(), None);
        assert_eq!(custom_checks_from_toml.triggers_pht(), None);
    }
}
