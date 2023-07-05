use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::spanned::Spanned;
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
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = ast.data
    {
        fields
    } else {
        panic!("Only support Struct")
    };

    let mut field_id = Vec::new();
    let mut field_value = Vec::new();
    let mut types = Vec::new();

    for field in fields.named.iter() {
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
                    if let Some(field_val) = &self.#field_value {
                        println!("{}: {}", #field_id, field_val);
                        toml_string.push_str(&format!("{field_name} = {field_value}\n",
                        field_name = #field_id,
                        field_value = field_val
                        ));
                    } else {
                        let type_name = stringify!(#types);
                        let mut type_as_char = type_name.chars();
                        for _ in 0..=7 {type_as_char.next();}
                        type_as_char.next_back();
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
