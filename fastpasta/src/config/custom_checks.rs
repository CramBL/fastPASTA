//! [CustomChecksOpt] trait for getting expected counter values to compare against the data.`
//! [CustomChecks] struct generated by deserializing TOML with expected counter values.

pub mod custom_checks_cfg;

use crate::util::*;

/// Trait for the configuration of various expected counters in the data.
pub trait CustomChecksOpt {
    /// Get a reference to the [CustomChecks] struct, if it is initialized
    fn custom_checks(&'static self) -> Option<&'static CustomChecks>;

    /// Returns if any custom checks are enabled.
    fn custom_checks_enabled(&'static self) -> bool;

    /// Returns if the option to generate a TOML file with default custom checks is enabled.
    fn generate_custom_checks_toml_enabled(&self) -> bool;

    /// Get the custom checks from a TOML file returning a [CustomChecks] instance.
    fn custom_checks_from_path(&self, toml_path: &PathBuf) -> CustomChecks {
        let toml = fs::read_to_string(toml_path).expect("Failed to read TOML file");
        toml::from_str(&toml).expect("Failed to parse TOML")
    }

    /// Write a [CustomChecks] instance to a TOML file in the current directory.
    fn generate_custom_checks_toml(&self, file_name: &str) {
        let toml =
            descriptive_toml_derive::TomlConfig::to_string_pretty_toml(&CustomChecks::default());
        fs::write(file_name, toml).expect("Failed to write TOML file");
    }

    /// Get the number of CDPs expected in the data, if it is set.
    fn cdps(&'static self) -> Option<u32>;

    /// Get the number of sent Triggers expected in the data, if it is set.
    fn triggers_pht(&'static self) -> Option<u32>;

    /// Get the expected RDH version, if it is set.
    fn rdh_version(&'static self) -> Option<u8>;

    /// Get the chip orders expected in the data, if it is set.
    ///
    /// Returns a slice of vectors, representing the legal chip orders for the Outer Barrel (ML/OL).
    fn chip_orders_ob(&'static self) -> Option<&[Vec<u8>]>;

    /// Get the number of chips expected in the data from Outer Barrel (ML/OL), if it is set.
    fn chip_count_ob(&'static self) -> Option<u8>;
}

impl<T> CustomChecksOpt for &T
where
    T: CustomChecksOpt,
{
    fn custom_checks(&'static self) -> Option<&'static CustomChecks> {
        (*self).custom_checks()
    }

    fn custom_checks_enabled(&'static self) -> bool {
        (*self).custom_checks_enabled()
    }

    fn generate_custom_checks_toml_enabled(&self) -> bool {
        (*self).generate_custom_checks_toml_enabled()
    }

    fn cdps(&'static self) -> Option<u32> {
        (*self).cdps()
    }

    fn triggers_pht(&'static self) -> Option<u32> {
        (*self).triggers_pht()
    }

    fn rdh_version(&'static self) -> Option<u8> {
        (*self).rdh_version()
    }

    fn chip_orders_ob(&'static self) -> Option<&'static [std::vec::Vec<u8>]> {
        (*self).chip_orders_ob()
    }

    fn chip_count_ob(&'static self) -> Option<u8> {
        (*self).chip_count_ob()
    }
}

impl<T> CustomChecksOpt for Box<T>
where
    T: CustomChecksOpt,
{
    fn custom_checks(&'static self) -> Option<&'static CustomChecks> {
        (**self).custom_checks()
    }

    fn custom_checks_enabled(&'static self) -> bool {
        (**self).custom_checks_enabled()
    }

    fn generate_custom_checks_toml_enabled(&self) -> bool {
        (**self).generate_custom_checks_toml_enabled()
    }

    fn cdps(&'static self) -> Option<u32> {
        (**self).cdps()
    }

    fn triggers_pht(&'static self) -> Option<u32> {
        (**self).triggers_pht()
    }

    fn rdh_version(&'static self) -> Option<u8> {
        (**self).rdh_version()
    }

    fn chip_orders_ob(&'static self) -> Option<&[Vec<u8>]> {
        (**self).chip_orders_ob()
    }

    fn chip_count_ob(&'static self) -> Option<u8> {
        (**self).chip_count_ob()
    }
}

impl<T> CustomChecksOpt for Arc<T>
where
    T: CustomChecksOpt,
{
    fn custom_checks(&'static self) -> Option<&'static CustomChecks> {
        (**self).custom_checks()
    }

    fn custom_checks_enabled(&'static self) -> bool {
        (**self).custom_checks_enabled()
    }

    fn generate_custom_checks_toml_enabled(&self) -> bool {
        (**self).generate_custom_checks_toml_enabled()
    }

    fn cdps(&'static self) -> Option<u32> {
        (**self).cdps()
    }

    fn triggers_pht(&'static self) -> Option<u32> {
        (**self).triggers_pht()
    }

    fn rdh_version(&'static self) -> Option<u8> {
        (**self).rdh_version()
    }

    fn chip_orders_ob(&'static self) -> Option<&[Vec<u8>]> {
        (**self).chip_orders_ob()
    }

    fn chip_count_ob(&'static self) -> Option<u8> {
        (**self).chip_count_ob()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_toml_to_file_trait() {
        let mock_file_name = "mock_test_custom_checks.toml";
        let mock_config = MockConfig::default();
        mock_config.generate_custom_checks_toml(mock_file_name);
        // Cleanup
        fs::remove_file(mock_file_name).unwrap();
    }
}
