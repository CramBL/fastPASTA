/// This trait is derived through the [TomlConfig](fastpasta_toml_macro_derive::TomlConfig) derive macro.
pub trait TomlConfig {
    /// Generates a customized pretty [String] representation of the serialized struct as a `TOML` template.
    /// The template includes comments with all possible fields and their types, that is easily edited and deserializes into the struct it was serialized from.
    /// The template also includes comments with descriptions, and examples.
    fn to_string_pretty_toml(&self) -> String;
}
