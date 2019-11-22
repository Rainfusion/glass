//! glass-derive provides proc-macros for the main library glass.
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Indexable)]
pub fn impl_indexable(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);

    // Match on the parsed item and respond accordingly.
    let output = match item.data {
        Data::Struct(ref struct_item) => match struct_item.fields {
            Fields::Named(ref fields) => fields
                .named
                .iter()
                .map(|field| field.clone().ident.unwrap().to_string())
                .collect(),
            _ => vec![],
        },
        // If the attribute was applied to any other kind of item, we want
        // to generate a compiler error.
        _ => panic!("Sorry, Indexable is not implemented for union or enum type."),
    };

    let name = &item.ident;

    let generated = quote! {
        impl #name {
            pub fn fields() -> &'static [&'static str] {
                &[#(#output),*]
            }
        }
    };

    generated.into()
}
