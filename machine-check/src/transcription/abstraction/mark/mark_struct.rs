use proc_macro2::Span;
use quote::quote;
use syn::{
    token::{Bracket, Paren},
    Attribute, ItemStruct, MetaList, Token,
};
use syn_path::path;

use super::mark_type_path::TypePathVisitor;

pub fn transcribe_struct(s: &ItemStruct) -> anyhow::Result<ItemStruct> {
    let mut mark_s = s.clone();
    mark_s.attrs.push(Attribute {
        pound_token: Token![#](Span::call_site()),
        style: syn::AttrStyle::Outer,
        bracket_token: Bracket::default(),
        meta: syn::Meta::List(MetaList {
            path: path![derive],
            delimiter: syn::MacroDelimiter::Paren(Paren::default()),
            tokens: quote!(Default),
        }),
    });
    TypePathVisitor::new().visit_struct(&mut mark_s);
    Ok(mark_s)
}
