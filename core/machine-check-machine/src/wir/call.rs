use machine_check_common::{ir_common::IrTypeArray, Signedness};
use proc_macro2::Span;
use std::fmt::Debug;
use syn::{
    punctuated::Punctuated, token::Paren, Expr, ExprCall, ExprLit, ExprPath, Lit, LitBool, LitInt,
};

use crate::{
    util::{create_expr_ident, create_expr_path, path_matches_global_names},
    wir::WSpan,
};

use super::{IntoSyn, WIdent, WMckBinary, WMckUnary, WPath, WPathSegment, WStdBinary, WStdUnary};

#[derive(Clone, Debug, Hash)]
pub enum WExprHighCall {
    Call(WCall),
    StdUnary(WStdUnary),
    StdBinary(WStdBinary),
    MckExt(WHighMckExt),
    MckNew(WHighMckNew),
    BooleanNew(bool),
    StdInto(WHighStdInto),
    StdClone(WIdent),
    ArrayRead(WArrayRead),
    ArrayWrite(WArrayWrite),
    Phi(WPhi),
    PhiTaken(WPhiTaken),
    PhiNotTaken,
}

#[derive(Clone, Hash)]
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
}

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
    pub fn_path: WPath,
    pub args: Vec<WCallArg>,
}

#[derive(Clone, Hash)]
pub enum WCallArg {
    Ident(WIdent),
    Literal(Lit),
}

impl IntoSyn<Expr> for WExprCall {
    fn into_syn(self) -> Expr {
        let (fn_path, args) = self.call_fn_and_args();
        WCall { fn_path, args }.into_syn()
    }
}

impl WExprCall {
    pub fn call_fn_and_args(self) -> (WPath, Vec<WCallArg>) {
        let span = Span::call_site();
        let (fn_operand, args) = match self {
            WExprCall::Call(call) => return (call.fn_path, call.args),
            WExprCall::MckUnary(call) => {
                let operation = call.op.to_string();
                (operation, vec![WCallArg::Ident(call.operand)])
            }
            WExprCall::MckBinary(call) => {
                let operation = call.op.to_string();
                (
                    operation,
                    vec![WCallArg::Ident(call.a), WCallArg::Ident(call.b)],
                )
            }
            WExprCall::MckExt(call) => (
                String::from(match call.signed {
                    false => MCK_UEXT,
                    true => MCK_SEXT,
                }),
                vec![WCallArg::Ident(call.from)],
            ),
            WExprCall::MckNew(call) => match call {
                WMckNew::BitvectorArray(_type_array, ident) => (
                    String::from(MCK_BITVECTOR_ARRAY_NEW),
                    vec![WCallArg::Ident(ident)],
                ),
                WMckNew::Bitvector(bitvector) => (
                    String::from(MCK_BITVECTOR_NEW),
                    vec![WCallArg::Literal(Lit::Int(LitInt::new(
                        bitvector.to_u64().to_string().as_str(),
                        span,
                    )))],
                ),
            },
            WExprCall::BooleanNew(value) => (
                String::from(BOOLEAN_NEW),
                vec![WCallArg::Literal(Lit::Bool(LitBool { value, span }))],
            ),
            WExprCall::StdClone(from) => (String::from(STD_CLONE), vec![WCallArg::Ident(from)]),
            WExprCall::ArrayRead(read) => (
                String::from(ARRAY_READ),
                vec![WCallArg::Ident(read.base), WCallArg::Ident(read.index)],
            ),
            WExprCall::ArrayWrite(write) => (
                String::from(ARRAY_WRITE),
                vec![
                    WCallArg::Ident(write.base),
                    WCallArg::Ident(write.index),
                    WCallArg::Ident(write.element),
                ],
            ),
            WExprCall::Phi(phi) => (
                String::from(PHI),
                vec![
                    WCallArg::Ident(phi.then_ident),
                    WCallArg::Ident(phi.else_ident),
                ],
            ),
            WExprCall::PhiTaken(taken) => (
                String::from(PHI_TAKEN),
                vec![
                    WCallArg::Ident(taken.ident),
                    WCallArg::Ident(taken.condition),
                ],
            ),
            WExprCall::PhiNotTaken => (String::from(PHI_NOT_TAKEN), vec![]),
        };
        (construct_call_fn_path(fn_operand), args)
    }

    pub fn right_idents(&self) -> Vec<WIdent> {
        let mut result = Vec::new();
        let (_call_fn, args) = self.clone().call_fn_and_args();
        for arg in args {
            match arg {
                WCallArg::Ident(ident) => result.push(ident),
                WCallArg::Literal(_) => {}
            }
        }
        result
    }

    pub fn replace_ident(&mut self, original: &WIdent, replacement: &WIdent) {
        match self {
            WExprCall::Call(call) => {
                for arg in &mut call.args {
                    match arg {
                        WCallArg::Ident(ident) => {
                            replace_ident(ident, original, replacement);
                        }
                        WCallArg::Literal(_) => {}
                    }
                }
            }
            WExprCall::MckUnary(mck_unary) => {
                replace_ident(&mut mck_unary.operand, original, replacement);
            }
            WExprCall::MckBinary(mck_binary) => {
                replace_ident(&mut mck_binary.a, original, replacement);
                replace_ident(&mut mck_binary.b, original, replacement);
            }
            WExprCall::MckExt(mck_ext) => {
                replace_ident(&mut mck_ext.from, original, replacement);
            }
            WExprCall::MckNew(mck_new) => match mck_new {
                WMckNew::Bitvector(_) => {}
                WMckNew::BitvectorArray(_arr, ident) => {
                    replace_ident(ident, original, replacement);
                }
            },
            WExprCall::BooleanNew(_) => {}
            WExprCall::StdClone(ident) => {
                replace_ident(ident, original, replacement);
            }
            WExprCall::ArrayRead(array_read) => {
                replace_ident(&mut array_read.base, original, replacement);
                replace_ident(&mut array_read.index, original, replacement);
            }
            WExprCall::ArrayWrite(array_write) => {
                replace_ident(&mut array_write.base, original, replacement);
                replace_ident(&mut array_write.element, original, replacement);
                replace_ident(&mut array_write.index, original, replacement);
            }
            WExprCall::Phi(phi) => {
                replace_ident(&mut phi.condition, original, replacement);
                replace_ident(&mut phi.then_ident, original, replacement);
                replace_ident(&mut phi.else_ident, original, replacement);
            }
            WExprCall::PhiTaken(phi_taken) => {
                replace_ident(&mut phi_taken.ident, original, replacement);
                replace_ident(&mut phi_taken.condition, original, replacement);
            }
            WExprCall::PhiNotTaken => {}
        }
    }
}

fn replace_ident(ident: &mut WIdent, original: &WIdent, replacement: &WIdent) {
    if ident == original {
        *ident = replacement.clone();
    }
}

impl IntoSyn<Expr> for WExprHighCall {
    fn into_syn(self) -> Expr {
        let span = Span::call_site();
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
            WExprHighCall::MckExt(call) => {
                (String::from(MCK_HIGH_EXT), vec![WCallArg::Ident(call.from)])
            }
            WExprHighCall::MckNew(call) => match call {
                WHighMckNew::BitvectorArray(_type_array, ident) => (
                    String::from(MCK_HIGH_BITVECTOR_ARRAY_NEW),
                    vec![WCallArg::Ident(ident)],
                ),
                WHighMckNew::Bitvector(signedness, _width, constant) => (
                    String::from(match signedness {
                        Signedness::None => MCK_HIGH_BITVECTOR_NEW,
                        Signedness::Unsigned => MCK_HIGH_UNSIGNED_NEW,
                        Signedness::Signed => MCK_HIGH_SIGNED_NEW,
                    }),
                    vec![WCallArg::Literal(Lit::Int(LitInt::new(
                        constant.to_string().as_str(),
                        span,
                    )))],
                ),
            },
            WExprHighCall::BooleanNew(value) => (
                String::from(BOOLEAN_NEW),
                vec![WCallArg::Literal(Lit::Bool(LitBool { value, span }))],
            ),
            WExprHighCall::StdInto(call) => {
                (String::from(STD_INTO), vec![WCallArg::Ident(call.from)])
            }
            WExprHighCall::StdClone(from) => (String::from(STD_CLONE), vec![WCallArg::Ident(from)]),
            WExprHighCall::ArrayRead(read) => (
                String::from(ARRAY_READ),
                vec![WCallArg::Ident(read.base), WCallArg::Ident(read.index)],
            ),
            WExprHighCall::ArrayWrite(write) => (
                String::from(ARRAY_WRITE),
                vec![
                    WCallArg::Ident(write.base),
                    WCallArg::Ident(write.index),
                    WCallArg::Ident(write.element),
                ],
            ),
            WExprHighCall::Phi(phi) => (
                String::from(PHI),
                vec![
                    WCallArg::Ident(phi.then_ident),
                    WCallArg::Ident(phi.else_ident),
                ],
            ),
            WExprHighCall::PhiTaken(taken) => (
                String::from(PHI_TAKEN),
                vec![
                    WCallArg::Ident(taken.ident),
                    WCallArg::Ident(taken.condition),
                ],
            ),
            WExprHighCall::PhiNotTaken => (String::from(PHI_NOT_TAKEN), vec![]),
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

fn construct_call_fn_path(fn_operand: String) -> WPath {
    let span = Span::call_site();

    // construct the WPath
    let without_leading = fn_operand
        .strip_prefix("::")
        .expect("Special function operand should have a leading prefix");
    let segments: Vec<WPathSegment> = without_leading
        .split("::")
        .map(|segment| WPathSegment {
            ident: WIdent::new(String::from(segment), span),
        })
        .collect();
    WPath {
        leading_colon: Some(WSpan::from_span(span)),
        segments,
    }
}

impl Debug for WExprCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (call_fn, args) = self.clone().call_fn_and_args();

        write!(f, "{:?}", call_fn)?;

        let mut franz = f.debug_tuple("");

        for arg in &args {
            franz.field(arg);
        }

        franz.finish()
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
