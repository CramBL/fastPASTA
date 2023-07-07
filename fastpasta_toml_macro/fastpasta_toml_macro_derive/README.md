# Description
Procedural derive macro for serializing a struct into a TOML template with field descriptions that is easily edited and deserialized.

Nested structs are not currently supported.

# Purpose
Make it easy to write a struct that defines a `TOML` template for optional configuration of an executable. Once the struct is deserialized with the derive macro implemented `to_string_pretty_toml()` function, it can be written to a (TOML) file, the file should be understandable without knowing any details of the binary. Deserializing the produced TOML file with no edits produceses the original struct with all optional fields `None`. Editing the produced TOML file will then deserialize into the original struct with those edited values.

# What is derived?

A `pub trait` named `TomlConfig` with a single function with the signature:  `fn to_string_pretty_toml(&self) -> String`

```rust
pub trait TomlConfig {
    fn to_string_pretty_toml(&self) -> String;
}
```
# Example use in fastPASTA
This macro was originally made for use in the [fastPASTA](https://crates.io/crates/fastpasta) crate.

This example is based on how the macro is used in `fastPASTA`.

The a struct named `CustomChecks` is implemented like this:

```rust
use fastpasta_toml_macro_derive::TomlConfig;
use serde_derive::{Deserialize, Serialize};

pub trait TomlConfig {
    fn to_string_pretty_toml(&self) -> String;
}

// Deriving the `TomlConfig` macro which implements the `TomlConfig` trait.
#[derive(TomlConfig, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomChecks {
    // Use the `description` field attribute of the macro
    #[description = "Number of CRU Data Packets expected in the data"]
    // Use the `example` field attribute of the macro to show some example values
    #[example = "20, 500532"]
    cdps: Option<u32>,

    #[description = "Number of Physics (PhT) Triggers expected in the data"]
    #[example = "0, 10"]
    triggers_pht: Option<u32>,

    #[description = "Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of two lists of 7 chip IDs"]
    #[example = "[[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]"]
    chip_orders_ob: Option<(Vec<u8>, Vec<u8>)>,
}
```

The template file is generated e.g. like this.
```rust
let toml = CustomChecks::default().to_string_pretty_toml();
std::fs::write("custom_checks.toml", toml).unwrap();
```
The contents of "custom_checks.toml" is now:
```toml
# Number of CRU Data Packets expected in the data
# Example: 20, 500532
#cdps = None [ u32 ] # (Uncomment and set to enable this check)

# Number of Physics (PhT) Triggers expected in the data
# Example: 0, 10
#triggers_pht = None [ u32 ] # (Uncomment and set to enable this check)

# Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of two lists of 7 chip IDs
# Example: [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]
#chip_orders_ob = None [ (Vec < u8 >, Vec < u8 >) ] # (Uncomment and set to enable this check)
```

Editing all the fields to contain `Some` values could look like this:
```toml
# Number of CRU Data Packets expected in the data
# Example: 20, 500532
cdps = 20

# Number of Physics (PhT) Triggers expected in the data
# Example: 0, 10
triggers_pht = 0

# Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of two lists of 7 chip IDs
# Example: [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]
chip_orders_ob = [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]
```
Deserializing from a TOML file is the same method as with any other TOML file, using `serde_derive`:
```rust
let toml = std::fs::read_to_string("custom_checks.toml").unwrap();
let custom_checks = toml::from_str(&toml).unwrap();
```

A user that is already familiar with the configuration file might simply write
```toml
cdps = 10
```
And input it to the binary. Which would deserialize into a struct with the `cdps` field containing `Some(10)`, and the rest of the fields are `None`.
