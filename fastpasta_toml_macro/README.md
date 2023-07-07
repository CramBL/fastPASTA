# Description
Convenience crate with a trait definition for use with the procedural derive macro `fastpasta_toml_macro_derive`.

# Example

```rust
use fastpasta_toml_macro::TomlConfig;

#[derive(TomlConfig, Default)]
pub struct CustomChecks {
    #[description = "Number of CRU Data Packets expected in the data"]
    #[example = "20, 500532"]
    cdps: Option<u32>,
}
```

```rust
let toml_string = CustomChecks::default().to_string_pretty_toml();
        assert_eq!(
            toml_string,
            "\
# Number of CRU Data Packets expected in the data
# Example: 20, 500532
#cdps = None [ u32 ] # (Uncomment and set to enable this check)

"
```
