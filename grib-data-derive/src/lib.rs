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
    let variant_names = variants.into_iter().map(|v| v.ident.clone());
    let variant_descriptions = variants
        .into_iter()
        .map(|v| {
            let desc_attribute = v.attrs.iter().find(|a| a.path.is_ident("description"));
            match desc_attribute {
                Some(a) => a.tokens.to_string().replace("=", "").replace("\"", "").trim().to_string(),
                _ => v.ident.to_string().to_lowercase(),
            }
        });

    (quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let description = match self {
                    #(
                        #name::#variant_names => #variant_descriptions,
                    )*
                };
                write!(f, "{}", description)
            }
        }
    }).into()
}

#[proc_macro_derive(FromValue)]
pub fn from_value(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let item: Item = input.into();

    if let Item::Enum(e) = item {
        // Build the output, possibly using quasi-quotation
        let expanded = generate_from_value_impl(&e);

        // Hand the output tokens back to the compiler
        TokenStream::from(expanded)
    } else {
        panic!("Only Enums are supported for DisplayDescription!");
    }
}

fn generate_from_value_impl(enum_data: &ItemEnum) -> TokenStream {
    let name: &syn::Ident = &enum_data.ident;
    let variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma> = &enum_data.variants;
    let variant_names = variants.into_iter().map(|v| v.ident.clone());
    let default_variant_name = variant_names.clone().last().clone().unwrap();
    let variant_values = variants.into_iter().map(|v| match &v.discriminant {
        Some((_, expr)) => match expr {
            syn::Expr::Lit(value) => match &value.lit {
                syn::Lit::Int(i) => i.base10_parse().unwrap_or(254u8),
                _ => 253u8,
            },
            _ => 252u8
        },
        None => 251u8,
    });

    (quote! {
        impl std::convert::From<u8> for #name {
            fn from(value: u8) -> Self {
                match value {
                    #(
                        #variant_values => #name::#variant_names,
                    )*
                    _ => #name::#default_variant_name
                }
            }
        }
    }).into()
}

#[proc_macro_derive(Parameter, attributes(abbrev, unit))]
pub fn parameter_attributes(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let item: Item = input.into();

    if let Item::Enum(e) = item {
        // Build the output, possibly using quasi-quotation
        let expanded = generate_parameter_attributes(&e);

        println!("{}", expanded);

        // Hand the output tokens back to the compiler
        TokenStream::from(expanded)
    } else {
        panic!("Only Enums are supported for DisplayDescription!");
    }
}

fn generate_parameter_attributes(enum_data: &ItemEnum) -> TokenStream {
    let name: &syn::Ident = &enum_data.ident;
    let variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma> = &enum_data.variants;
    let variant_names_first = variants.into_iter().map(|v| v.ident.clone());
    let variant_names_second = variants.into_iter().map(|v| v.ident.clone());
    let variant_abbreviations = variants
        .into_iter()
        .map(|v| {
            let abbrev_attribute = v.attrs.iter().find(|a| a.path.is_ident("abbrev"));
            match abbrev_attribute {
                Some(a) => a.tokens.to_string().replace("=", "").replace("\"", "").trim().to_string(),
                _ => v.ident.to_string().to_lowercase(),
            }
        });
    let variant_units = variants
        .into_iter()
        .map(|v| {
            let unit_attribute = v.attrs.iter().find(|a| a.path.is_ident("unit"));
            match unit_attribute {
                Some(a) => a.tokens.to_string().replace("=", "").replace("\"", "").trim().to_string(),
                _ => v.ident.to_string().to_lowercase(),
            }
        });

    (quote! {
        impl #name {
            fn abbrev(&self) -> &str {
                match self {
                    #(
                        #name::#variant_names_first => #variant_abbreviations,
                    )*
                }
            }

            fn unit(&self) -> &str {
                match self {
                    #(
                        #name::#variant_names_second => #variant_units,
                    )*
                }
            }
        }
    }).into()
}