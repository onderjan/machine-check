use machine_check_common::{ir_common::IrTypeArray, Signedness};
use proc_macro2::Span;
use std::fmt::Debug;
use syn::{
    punctuated::Punctuated,
    token::{Comma, Paren},
    Expr, ExprCall, ExprInfer, ExprLit, ExprPath, Ident, Lit, LitInt, Path, PathSegment, Token,
    Type,
};

use crate::wir::{IntoSyn, WTypeId};

use super::{WCall, WIdent, WMckBinary, WMckUnary};

#[derive(Clone, Hash, Debug)]
pub enum WExprLowCall {
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

impl IntoSyn<Expr> for WExprLowCall {
    fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Expr {
        let span = Span::call_site();

        let (func_path, func_args) = match self {
            WExprLowCall::Call(call) => {
                return call.into_syn(type_fn);
                /*let func = call.fn_path.into();
                let args = Punctuated::from_iter(call.args.iter().map(|arg| arg.into() ));
                (func, args)*/
            }
            WExprLowCall::MckUnary(unary) => todo!("unary"),
            WExprLowCall::MckBinary(binary) => (
                binary.op.to_string(),
                convert_args(vec![binary.a, binary.b]),
            ),
            WExprLowCall::MckExt(ext) => todo!("ext"),
            WExprLowCall::MckNew(new) => {
                let span = Span::call_site();
                match new {
                    WMckNew::Bitvector(concrete_bitvector) => (
                        MCK_BITVECTOR_NEW.to_string(),
                        Punctuated::<Expr, Comma>::from_iter([
                            Expr::Lit(ExprLit {
                                attrs: Vec::new(),
                                lit: Lit::Int(LitInt::new(
                                    &concrete_bitvector.to_u64().to_string(),
                                    span,
                                )),
                            }),
                            Expr::Infer(ExprInfer {
                                attrs: Vec::new(),
                                underscore_token: Token![_](span),
                            }),
                        ]),
                    ),
                    WMckNew::BitvectorArray(ir_type_array, wident) => todo!("Mck array"),
                }
            }
            WExprLowCall::BooleanNew(value) => todo!("value"),
            WExprLowCall::StdClone(ident) => todo!("ident"),
            WExprLowCall::ArrayRead(array_read) => todo!("array read"),
            WExprLowCall::ArrayWrite(array_write) => todo!("array write"),
            WExprLowCall::Phi(phi) => (
                PHI.to_string(),
                convert_args(vec![phi.then_ident, phi.else_ident]),
            ),
            WExprLowCall::PhiTaken(phi_taken) => (
                PHI_TAKEN.to_string(),
                convert_args(vec![phi_taken.ident, phi_taken.condition]),
            ),
            WExprLowCall::PhiNotTaken => (PHI_NOT_TAKEN.to_string(), convert_args(vec![])),
        };

        let func_path = func_path
            .strip_prefix("::")
            .expect("Expected absolute path");
        let mut path = Path {
            leading_colon: Some(Token![::](span)),
            segments: Punctuated::new(),
        };
        for segment in func_path.split("::") {
            path.segments.push(PathSegment {
                ident: Ident::new(segment, span),
                arguments: syn::PathArguments::None,
            });
        }
        Expr::Call(ExprCall {
            attrs: Vec::new(),
            func: Box::new(Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path,
            })),
            paren_token: Paren::default(),
            args: func_args,
        })
    }
}

fn convert_args(func_args: Vec<WIdent>) -> Punctuated<Expr, Comma> {
    let mut args = Punctuated::new();

    for arg in func_args {
        args.push(Expr::Path(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: Path::from(Ident::from(arg)),
        }));
    }
    args
}
