use std::fmt::Debug;
use syn::{punctuated::Punctuated, token::Paren, Expr, ExprCall, ExprLit, ExprPath, Lit, Type};

use crate::{
    util::{create_expr_ident, create_expr_path, path_matches_global_names},
    wir::{WPartialPath, WPartialPathSegment, WSpan, WTypeId},
};

use super::{IntoTypedSyn, WIdent, WMckBinary, WMckUnary, WStdBinary, WStdUnary};

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

impl IntoTypedSyn<Expr> for WCall {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Expr {
        let path = self.fn_path.into_typed_syn(&type_fn);

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
    let span = WSpan::call_site();
    let without_leading = fn_operand
        .strip_prefix("::")
        .expect("Special function operand should have a leading prefix");
    let segments: Vec<WPartialPathSegment> = without_leading
        .split("::")
        .map(|segment| WPartialPathSegment {
            ident: WIdent::new(String::from(segment), span),
            generics: None,
        })
        .collect();
    WPartialPath {
        leading_colon: Some(span),
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
