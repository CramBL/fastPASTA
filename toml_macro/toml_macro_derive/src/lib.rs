use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::MetaList;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TomlConfig, attributes(description, example))]
pub fn derive_signature(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse_macro_input!(input as DeriveInput);
    // Build the trait implementation
    impl_toml_config(&ast)
}

fn impl_toml_config(ast: &syn::DeriveInput) -> TokenStream {
    const DESCRIPTION_ATTR_NAME: &'static str = "description";
    const EXAMPLE_ATTR_NAME: &'static str = "example";

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
        let description: Option<&syn::Attribute> = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident(DESCRIPTION_ATTR_NAME));
        if let Some(desc) = description {
            let attr_description_as_string = desc
                .meta
                .require_name_value()
                .unwrap()
                .value
                .to_token_stream()
                .to_string();
            let mut as_char_iter = attr_description_as_string.chars();
            as_char_iter.next();
            as_char_iter.next_back();
            descriptions.push(as_char_iter.as_str().to_owned());
        } else {
            panic!("Every custom check field needs a description!")
        }

        let example: Option<&syn::Attribute> = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident(EXAMPLE_ATTR_NAME));
        if let Some(example) = example {
            let attr_example_as_string = example
                .meta
                .require_name_value()
                .unwrap()
                .value
                .to_token_stream()
                .to_string();
            let mut as_char_iter = attr_example_as_string.chars();
            as_char_iter.next();
            as_char_iter.next_back();
            examples.push(as_char_iter.as_str().to_owned());
        } else {
            panic!("Every custom check field needs an example!")
        }

        let field_name: &syn::Ident = field.ident.as_ref().unwrap();
        let name: String = field_name.to_string();
        let literal_key_str = syn::LitStr::new(&name, field.span());
        let type_name = &field.ty;
        field_id.push(quote! { #literal_key_str });
        field_value.push(&field.ident);
        types.push(type_name.to_token_stream());
    }

    let name = &ast.ident;
    let gen = quote! {
        impl TomlConfig for #name {
            fn to_string_pretty_toml(&self) -> String {
                let name = stringify!(#name);
                let mut toml_string = String::from(&format!("[{}]\n", name));

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
                        println!("{}: {}", #field_id, field_val);
                        let formatted_field_val = if is_type_string {
                                format!("\"{field_val}\"")
                        } else {
                                format!("{field_val}")
                        };
                        toml_string.push_str(&format!("{field_name} = {field_value} # [{type_name}]\n",
                            field_name = #field_id,
                            field_value = formatted_field_val,
                            type_name = type_as_char.as_str()
                        ));
                    } else {

                        println!("#{}: None [{type_name}] # (Uncomment and set to enable this check)",
                        #field_id,
                        type_name = type_as_char.as_str());
                        toml_string.push_str(&format!("#{field_name} = None [{type_name}] # (Uncomment and set to enable this check)\n",
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
