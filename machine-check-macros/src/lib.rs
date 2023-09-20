#![feature(log_syntax)]

use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Abstraction)]
pub fn derive_abstraction(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let concrete_ident = input.ident.clone();

    dbg!("Hello there");
    let abstract_name = format!("__machine_check_{}", input.ident.to_string());
    let abstract_ident = Ident::new(&abstract_name, Span::call_site());

    let struct_data = match input.data {
        syn::Data::Struct(data_struct) => data_struct,
        syn::Data::Enum(_) => {
            panic!("Enums currently not supported!")
        }
        syn::Data::Union(_) => {
            panic!("Unions not supported!")
        }
    };

    let fields = match struct_data.fields {
        syn::Fields::Named(fields_named) => fields_named,
        _ => {
            panic!("Only named fields supported!");
        }
    };

    let mut abstract_fields_named = fields.named;

    for abstract_field in &mut abstract_fields_named {
        let ty = &abstract_field.ty;

        let abstract_type: syn::Type = syn::parse_quote!(
            crate::interval_domain::IntervalDomain< #ty >
        );
        abstract_field.ty = abstract_type;
    }

    // Build the output using quasi-quotation
    let expanded = quote! {
        struct #abstract_ident {
            #abstract_fields_named
        }
        impl ::machine_check_traits::Abstraction for #concrete_ident {
            type AbstractType = #abstract_ident;
        }
    };
    dbg!(expanded.to_string());

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}
