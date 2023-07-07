//! # Description
//! Procedural derive macro for serializing a struct into a TOML template with field descriptions that is easily edited and deserialized.
//!
//! Nested structs are not currently supported.
//!
//! # Purpose
//! Make it easy to write a struct that defines a `TOML` template for optional configuration of an executable. Once the struct is deserialized with the derive macro implemented `to_string_pretty_toml()` function, it can be written to a (TOML) file, the file should be understandable without knowing any details of the binary. Deserializing the produced TOML file with no edits produceses the original struct with all optional fields `None`. Editing the produced TOML file will then deserialize into the original struct with those edited values.
//!
//! # Table of Contents
//! - [Description](#description)
//! - [Purpose](#purpose)
//! - [Table of Contents](#table-of-contents)
//! - [Guide](#guide)
//!   - [What is derived?](#what-is-derived)
//!   - [Example use in fastPASTA](#example-use-in-fastpasta)
//!     - [Implementing](#implementing)
//!     - [Serializing](#serializing)
//!     - [Deserializing](#deserializing)
//!
//! # Guide
//!
//! ## What is derived?
//! A `pub trait` named `TomlConfig` with a single function with the signature:  `fn to_string_pretty_toml(&self) -> String`
//!
//! ```rust
//! pub trait TomlConfig {
//!     fn to_string_pretty_toml(&self) -> String;
//! }
//! ```
//!
//! ## Example use in fastPASTA
//! This macro was originally made for use in the [fastPASTA](https://crates.io/crates/fastpasta) crate.
//! The example is based on how the macro is used in `fastPASTA`.
//!
//! ### Implementing
//! The struct `CustomChecks` is implemented like this:
//!
//! ```rust
//! use fastpasta_toml_macro_derive::TomlConfig;
//! use serde_derive::{Deserialize, Serialize};
//!
//! pub trait TomlConfig {
//!     fn to_string_pretty_toml(&self) -> String;
//! }
//!
//! // Deriving the `TomlConfig` macro which implements the `TomlConfig` trait.
//! #[derive(TomlConfig, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
//! pub struct CustomChecks {
//!     // Use the `description` field attribute of the macro
//!     #[description = "Number of CRU Data Packets expected in the data"]
//!     // Use the `example` field attribute of the macro to show some example values
//!     #[example = "20, 500532"]
//!     cdps: Option<u32>,
//!
//!     #[description = "Number of Physics (PhT) Triggers expected in the data"]
//!     #[example = "0, 10"]
//!     triggers_pht: Option<u32>,
//!
//!     #[description = "Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of two lists of 7 chip IDs"]
//!     #[example = "[[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]"]
//!     chip_orders_ob: Option<(Vec<u8>, Vec<u8>)>,
//! }
//! ```
//! ### Serializing
//!
//! The template file is generated e.g. like this.
//! ```rust
//! let toml = CustomChecks::default().to_string_pretty_toml();
//! std::fs::write("custom_checks.toml", toml).unwrap();
//! ```
//! The contents of "custom_checks.toml" is now:
//! ```toml
//! # Number of CRU Data Packets expected in the data
//! # Example: 20, 500532
//! #cdps = None [ u32 ] # (Uncomment and set to enable this check)
//!
//! # Number of Physics (PhT) Triggers expected in the data
//! # Example: 0, 10
//! #triggers_pht = None [ u32 ] # (Uncomment and set to enable this check)
//!
//! # Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of two lists of 7 chip IDs
//! # Example: [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]
//! #chip_orders_ob = None [ (Vec < u8 >, Vec < u8 >) ] # (Uncomment and set to enable this check)
//! ```
//! Editing all the fields to contain `Some` values could look like this:
//! ```toml
//! # Number of CRU Data Packets expected in the data
//! # Example: 20, 500532
//! cdps = 20
//!
//! # Number of Physics (PhT) Triggers expected in the data
//! # Example: 0, 10
//! triggers_pht = 0
//!
//! # Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of two lists of 7 chip IDs
//! # Example: [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]
//! chip_orders_ob = [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]
//! ```
//! ### Deserializing
//!
//! Deserializing from a TOML file is the same method as with any other TOML file, using `serde_derive`:
//! ```rust
//! let toml = std::fs::read_to_string("custom_checks.toml").unwrap();
//! let custom_checks = toml::from_str(&toml).unwrap();
//! ```
//!
//! A user that is already familiar with the configuration file might simply write
//! ```toml
//! cdps = 10
//! ```
//! And input it to the binary. Which would deserialize into a struct with the `cdps` field containing `Some(10)`, and the rest of the fields are `None`.

use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::{Attribute, DeriveInput};

#[proc_macro_derive(TomlConfig, attributes(description, example))]
pub fn derive_signature(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse_macro_input!(input as DeriveInput);
    // Build the trait implementation
    impl_toml_config(&ast)
}

fn impl_toml_config(ast: &syn::DeriveInput) -> TokenStream {
    const DESCRIPTION_ATTR_NAME: &str = "description";
    const EXAMPLE_ATTR_NAME: &str = "example";

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = ast.data
    {
        fields
    } else {
        panic!("This macro only works on structs")
    };

    let mut descriptions: Vec<String> = Vec::new();
    let mut examples: Vec<String> = Vec::new();
    let mut field_ids: Vec<quote::__private::TokenStream> = Vec::new();
    let mut field_values: Vec<&Option<syn::Ident>> = Vec::new();
    let mut types: Vec<String> = Vec::new();

    for field in fields.named.iter() {
        if let Some(desc) = get_attribute(DESCRIPTION_ATTR_NAME, field) {
            descriptions.push(attribute_value_as_string(desc));
        } else {
            panic!("Every custom check field needs a description!")
        }

        if let Some(example) = get_attribute(EXAMPLE_ATTR_NAME, field) {
            examples.push(attribute_value_as_string(example));
        } else {
            panic!("Every custom check field needs an example!")
        }

        let literal_key_str: syn::LitStr = field_name_to_key_literal(field.ident.as_ref().unwrap());
        field_ids.push(quote! { #literal_key_str  });

        field_values.push(&field.ident);
        types.push(field_option_type_to_inner_type_string(&field.ty));
    }

    let struct_name = &ast.ident;
    let generated_code_token_stream: quote::__private::TokenStream = generate_impl(
        struct_name,
        descriptions,
        examples,
        field_ids,
        field_values,
        types,
    );
    // Return the generated impl as a proc_macro::TokenStream instead of the TokenStream type that quote returns
    generated_code_token_stream.into()
}

/// Generate the implementation of the [TomlConfig] trait
fn generate_impl(
    struct_name: &syn::Ident,
    descriptions: Vec<String>,
    examples: Vec<String>,
    field_ids: Vec<quote::__private::TokenStream>,
    field_values: Vec<&Option<syn::Ident>>,
    types: Vec<String>,
) -> quote::__private::TokenStream {
    quote! {
        impl TomlConfig for #struct_name {
            fn to_string_pretty_toml(&self) -> String {

                let mut toml_string = String::new();

                #(
                    toml_string.push_str(&format!("# {description_comment}\n", description_comment = #descriptions));
                    toml_string.push_str(&format!("# Example: {example}\n", example = #examples));

                    if let Some(field_val) = &self.#field_values {
                         // If the type is `String` the value needs to be in quotes in TOML format
                        let formatted_field_val = if #types.contains(&"String") {
                                format!("\"{field_val:?}\"")
                        } else {
                            // If the type is a tuple struct, the value needs to be in square brackets in TOML format
                            // This is safe as we know by now that the type is not a `String`
                            let field_val_string = format!("{field_val:?}");
                            field_val_string.chars().map(|c| match c {
                                '(' => '[',
                                ')' => ']',
                                _ => c
                            }).collect::<String>()
                        };
                        toml_string.push_str(&format!("{field_name} = {field_value} # [{type_name}]\n\n",
                            field_name = #field_ids,
                            field_value = formatted_field_val,
                            type_name = #types
                        ));
                    } else {
                        toml_string.push_str(&format!("#{field_name} = None [{type_name}] # (Uncomment and set to enable)\n\n",
                            field_name = #field_ids,
                            type_name = #types
                        ));
                    }
                )*

                toml_string
            }
        }
    }
}

fn get_attribute<'a>(attr_name: &'a str, field: &'a syn::Field) -> Option<&'a syn::Attribute> {
    field.attrs.iter().find(|a| a.path().is_ident(attr_name))
}

fn attribute_value_as_string(attr: &Attribute) -> String {
    let attr_description_as_string = attr
        .meta
        .require_name_value()
        .unwrap()
        .value
        .to_token_stream()
        .to_string();
    let mut as_char_iter = attr_description_as_string.chars();
    as_char_iter.next();
    as_char_iter.next_back();
    as_char_iter.as_str().to_owned()
}

fn field_name_to_key_literal(field_name: &syn::Ident) -> syn::LitStr {
    let name: String = field_name.to_string();
    syn::LitStr::new(&name, field_name.span())
}

// Convert a fields type of type Option<InnerType> to InnerType as a string
fn field_option_type_to_inner_type_string(field_option_type: &syn::Type) -> String {
    let type_name = field_option_type.to_token_stream().to_string();
    let mut type_as_char = type_name.chars();
    for _ in 0..=7 {
        type_as_char.next();
    }
    type_as_char.next_back();
    type_as_char.as_str().to_string()
}
