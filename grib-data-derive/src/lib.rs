extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item, ItemEnum, DeriveInput};

#[proc_macro_derive(DisplayDescription, attributes(description))]
pub fn display_description(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let item: Item = input.into();

    if let Item::Enum(e) = item {
        // Build the output, possibly using quasi-quotation
        let expanded = generate_display_impl(&e);

        // Hand the output tokens back to the compiler
        TokenStream::from(expanded)
    } else {
        panic!("Only Enums are supported for DisplayDescription!");
    }
}

fn generate_display_impl(enum_data: &ItemEnum) -> TokenStream {
    let name: &syn::Ident = &enum_data.ident;
    let variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma> = &enum_data.variants;
    let variant_iter = variants.into_iter().map(|v| v.ident.clone());
    let variant_descriptions = variants
        .into_iter()
        .map(|v| {
            let desc_attribute = v.attrs.iter().find(|a| a.path.is_ident("description"));
            match desc_attribute {
                Some(a) => a.tokens.to_string().replace("=", "").replace("\"", "").trim().to_string(),
                _ => v.ident.to_string(),
            }
        });

    (quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let description = match self {
                    #(
                        #name::#variant_iter => #variant_descriptions,
                    )*
                };
                write!(f, "{}", description)
            }
        }
    }).into()
}