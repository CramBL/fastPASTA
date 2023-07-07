pub trait TomlConfig {
    fn to_string_pretty_toml(&self) -> String;
}
