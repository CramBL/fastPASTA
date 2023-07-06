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
    let mut field_id = Vec::new();
    let mut field_value = Vec::new();
    let mut types = Vec::new();

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

        let literal_key_str = field_name_to_key_literal(field.ident.as_ref().unwrap());
        field_id.push(quote! { #literal_key_str  });

        field_value.push(&field.ident);
        let type_name = &field.ty;
        types.push(type_name.to_token_stream());
    }

    let name = &ast.ident;
    let gen = quote! {
        impl TomlConfig for #name {
            fn to_string_pretty_toml(&self) -> String {

                let mut toml_string = String::new();

                #(

                    // Stringify the type. It will look like `Option < TYPE >`
                    let type_name = stringify!(#types);
                    // Remove the `Option< >` part of the string
                    let mut type_as_char = type_name.chars();
                    for _ in 0..=7 {type_as_char.next();}
                    type_as_char.next_back();
                    // Determine if the type is String as their value needs to be in quotes in TOML format
                    let is_type_string = type_as_char.as_str().contains(&"String");

                    toml_string.push_str(&format!("# {description_comment}\n", description_comment = #descriptions));
                    toml_string.push_str(&format!("# Example: {example}\n", example = #examples));
                    if let Some(field_val) = &self.#field_value {
                        //println!("{}: {}", #field_id, field_val);
                        let formatted_field_val = if is_type_string {
                                format!("\"{field_val}\"")
                        } else {
                                format!("{field_val}")
                        };
                        toml_string.push_str(&format!("{field_name} = {field_value} # [{type_name}]\n\n",
                            field_name = #field_id,
                            field_value = formatted_field_val,
                            type_name = type_as_char.as_str()
                        ));
                    } else {

                        // println!("#{}: None [{type_name}] # (Uncomment and set to enable this check)",
                        // #field_id,
                        // type_name = type_as_char.as_str());
                        toml_string.push_str(&format!("#{field_name} = None [{type_name}] # (Uncomment and set to enable this check)\n\n",
                            field_name = #field_id,
                            type_name = type_as_char.as_str()
                        ));
                    }
                )*


                toml_string
            }
        }
    };
    gen.into()
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
