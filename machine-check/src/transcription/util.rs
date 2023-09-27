pub mod path_rule;
pub mod scheme;

use proc_macro2::TokenStream;
use syn::{
    punctuated::Punctuated,
    token::{Bracket, Paren},
    Attribute, Expr, ExprCall, ExprPath, Ident, Local, LocalInit, MetaList, Pat, PatIdent, PatType,
    Stmt, Type,
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

pub fn generate_let_default_stmt(ident: Ident, ty: Type) -> Stmt {
    Stmt::Local(Local {
        attrs: vec![],
        let_token: Default::default(),
        pat: Pat::Type(PatType {
            attrs: vec![],
            pat: Box::new(Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: Some(Default::default()),
                ident,
                subpat: None,
            })),
            colon_token: Default::default(),
            ty: Box::new(ty),
        }),
        init: Some(LocalInit {
            eq_token: Default::default(),
            expr: Box::new(Expr::Call(ExprCall {
                attrs: vec![],
                func: Box::new(Expr::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: path!(::std::default::Default::default),
                })),
                paren_token: Default::default(),
                args: Punctuated::default(),
            })),
            diverge: None,
        }),
        semi_token: Default::default(),
    })
}
