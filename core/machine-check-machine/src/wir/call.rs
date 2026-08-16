use proc_macro2::Span;
use std::fmt::Debug;
use syn::{punctuated::Punctuated, token::Paren, Expr, ExprCall, ExprLit, ExprPath, Lit, Type};

use crate::{
    util::{create_expr_ident, create_expr_path, path_matches_global_names},
    wir::{WPartialPath, WPartialSegment, WSpan, WTypeId},
};

use super::{IntoSyn, WIdent, WMckBinary, WMckUnary, WStdBinary, WStdUnary};

mod high;
mod low;

pub use {high::WExprHighCall, low::*};

#[derive(Clone, Debug, Hash)]
pub struct WCall {
    pub fn_path: WPartialPath,
    pub args: Vec<WCallArg>,
}

#[derive(Clone, Hash)]
pub enum WCallArg {
    Ident(WIdent),
    Literal(Lit),
}

impl IntoSyn<Expr> for WCall {
    fn into_syn(self, _type_fn: &impl Fn(WTypeId) -> Type) -> Expr {
        let path = self.fn_path.into();

        let mut args = Punctuated::from_iter(self.args.into_iter().map(|arg| match arg {
            WCallArg::Ident(ident) => create_expr_ident(ident.into()),
            WCallArg::Literal(lit) => Expr::Lit(ExprLit {
                attrs: Vec::new(),
                lit,
            }),
        }));

        // kludge CBound
        if path_matches_global_names(&path, &["mck", "forward", "Bitvector", "new"]) {
            args.push(create_expr_path(syn_path::path!(::mck::misc::CBound)));
        }

        if path_matches_global_names(&path, &["mck", "forward", "Array", "new_filled"]) {
            args.insert(0, create_expr_path(syn_path::path!(::mck::misc::CBound)));
        }

        Expr::Call(ExprCall {
            attrs: Vec::new(),
            func: Box::new(Expr::Path(ExprPath {
                attrs: vec![],
                path,
                qself: None,
            })),
            paren_token: Paren::default(),
            args,
        })
    }
}

fn construct_call_fn_path(fn_operand: String) -> WPartialPath {
    let span = Span::call_site();
    let without_leading = fn_operand
        .strip_prefix("::")
        .expect("Special function operand should have a leading prefix");
    let segments: Vec<WPartialSegment> = without_leading
        .split("::")
        .map(|segment| WPartialSegment {
            ident: WIdent::new(String::from(segment), span),
            generics: None,
        })
        .collect();
    WPartialPath {
        leading_colon: Some(WSpan::from_span(span)),
        segments,
    }
}

impl Debug for WCallArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(ident) => ident.fmt(f),
            Self::Literal(lit) => write!(f, "{:?}", lit),
        }
    }
}
