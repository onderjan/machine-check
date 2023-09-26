use proc_macro2::{Span, TokenStream};
use syn::{
    token::{Bracket, Paren},
    Attribute, MetaList, Token,
};
use syn_path::path;

pub fn generate_derive_attribute(tokens: TokenStream) -> Attribute {
    Attribute {
        pound_token: Token![#](Span::call_site()),
        style: syn::AttrStyle::Outer,
        bracket_token: Bracket::default(),
        meta: syn::Meta::List(MetaList {
            path: path![derive],
            delimiter: syn::MacroDelimiter::Paren(Paren::default()),
            tokens,
        }),
    }
}
