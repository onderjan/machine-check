use proc_macro2::TokenStream;
use syn::{
    token::{Bracket, Paren},
    Attribute, MetaList,
};
use syn_path::path;

pub fn generate_derive_attribute(tokens: TokenStream) -> Attribute {
    Attribute {
        pound_token: Default::default(),
        style: syn::AttrStyle::Outer,
        bracket_token: Bracket::default(),
        meta: syn::Meta::List(MetaList {
            path: path![derive],
            delimiter: syn::MacroDelimiter::Paren(Paren::default()),
            tokens,
        }),
    }
}
