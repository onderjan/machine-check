use machine_check_common::{ir_common::IrTypeArray, Signedness};
use proc_macro2::Span;
use std::fmt::Debug;
use syn::{
    punctuated::Punctuated, token::Paren, Expr, ExprCall, ExprLit, ExprPath, Lit, LitBool, LitInt,
};

use crate::{
    util::{create_expr_ident, create_expr_path, path_matches_global_names},
    wir::{WPartialPath, WPartialSegment, WSpan},
};

use super::{IntoSyn, WIdent, WMckBinary, WMckUnary, WPath, WPathSegment, WStdBinary, WStdUnary};

#[derive(Clone, Debug, Hash)]
pub enum WExprHighCall {
    Call(WCall),
    StdUnary(WStdUnary),
    StdBinary(WStdBinary),
}

/*#[derive(Clone, Hash)]
pub enum WExprCall {
    Call(WCall),
    MckUnary(WMckUnary),
    MckBinary(WMckBinary),
    MckExt(WMckExt),
    MckNew(WMckNew),
    BooleanNew(bool),
    StdClone(WIdent),
    ArrayRead(WArrayRead),
    ArrayWrite(WArrayWrite),
    Phi(WPhi),
    PhiTaken(WPhiTaken),
    PhiNotTaken,
}*/

#[derive(Clone, Debug, Hash)]
pub struct WPhi {
    pub condition: WIdent,
    pub then_ident: WIdent,
    pub else_ident: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WPhiTaken {
    pub ident: WIdent,
    pub condition: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub enum WHighMckNew {
    Bitvector(Signedness, Option<u32>, i128),
    BitvectorArray(IrTypeArray, WIdent),
}

#[derive(Clone, Debug, Hash)]
pub enum WMckNew {
    Bitvector(mck::concr::ConcreteBitvector<mck::misc::RBound>),
    BitvectorArray(IrTypeArray, WIdent),
}

#[derive(Clone, Debug, Hash)]
pub struct WBitvectorNew {}

#[derive(Clone, Debug, Hash)]
pub struct WArrayNew {
    pub ty: IrTypeArray,
    pub fill_element: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WHighMckExt {
    pub width: Option<u32>,
    pub from: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WMckExt {
    pub signed: bool,
    pub width: u32,
    pub from: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WHighStdInto {
    pub signedness: Signedness,
    pub width: Option<u32>,
    pub from: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WArrayRead {
    pub base: WIdent,
    pub index: WIdent,
}
#[derive(Clone, Debug, Hash)]
pub struct WArrayWrite {
    pub base: WIdent,
    pub index: WIdent,
    pub element: WIdent,
}

pub const MCK_HIGH_EXT: &str = "::machine_check::Ext::ext";
pub const MCK_HIGH_BITVECTOR_NEW: &str = "::machine_check::Bitvector::new";
pub const MCK_HIGH_UNSIGNED_NEW: &str = "::machine_check::Unsigned::new";
pub const MCK_HIGH_SIGNED_NEW: &str = "::machine_check::Signed::new";
pub const MCK_HIGH_BITVECTOR_ARRAY_NEW: &str = "::machine_check::BitvectorArray::new_filled";

pub const BOOLEAN_NEW: &str = "::mck::forward::Boolean::new";

pub const MCK_UEXT: &str = "::mck::forward::Ext::uext";
pub const MCK_SEXT: &str = "::mck::forward::Ext::sext";
pub const MCK_BITVECTOR_NEW: &str = "::mck::forward::Bitvector::new";
pub const MCK_BITVECTOR_ARRAY_NEW: &str = "::mck::forward::Array::new_filled";

pub const STD_CLONE: &str = "::std::clone::Clone::clone";
pub const STD_INTO: &str = "::std::convert::Into::into";

pub const ARRAY_READ: &str = "::mck::forward::ReadWrite::read";
pub const ARRAY_WRITE: &str = "::mck::forward::ReadWrite::write";

pub const PHI: &str = "::mck::forward::PhiArg::phi";
pub const PHI_TAKEN: &str = "::mck::forward::PhiArg::Taken";
pub const PHI_NOT_TAKEN: &str = "::mck::forward::PhiArg::NotTaken";

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

fn replace_ident(ident: &mut WIdent, original: &WIdent, replacement: &WIdent) {
    if ident == original {
        *ident = replacement.clone();
    }
}

impl IntoSyn<Expr> for WExprHighCall {
    fn into_syn(self) -> Expr {
        let (fn_operand, args) = match self {
            WExprHighCall::Call(call) => return call.into_syn(),
            WExprHighCall::StdUnary(call) => {
                let operation = call.op.to_string();
                (operation, vec![WCallArg::Ident(call.operand)])
            }
            WExprHighCall::StdBinary(call) => {
                let operation = call.op.to_string();
                (
                    operation,
                    vec![WCallArg::Ident(call.a), WCallArg::Ident(call.b)],
                )
            }
        };
        let fn_path = construct_call_fn_path(fn_operand);
        WCall { fn_path, args }.into_syn()
    }
}

impl IntoSyn<Expr> for WCall {
    fn into_syn(self) -> Expr {
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
