use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, token::Paren, Expr, ExprCall, ExprLit, ExprPath, Lit, LitInt, Type,
};

use crate::util::create_expr_ident;

use super::{
    IntoSyn, WBasicType, WGeneric, WIdent, WMckBinary, WMckUnary, WPath, WPathSegment, WStdBinary,
    WStdUnary, WTypeArray,
};

#[derive(Clone, Debug, Hash)]
pub enum WExprHighCall<FT: IntoSyn<Type>> {
    Call(WCall<FT>),
    StdUnary(WStdUnary),
    StdBinary(WStdBinary),
    MckExt(WHighMckExt),
    MckNew(WHighMckNew),
    StdInto(WHighStdInto),
    StdClone(WIdent),
    ArrayRead(WArrayRead),
    ArrayWrite(WArrayWrite),
    Phi(WIdent, WIdent),
    PhiTaken(WIdent),
    PhiNotTaken,
    PhiUninit,
}

#[derive(Clone, Debug, Hash)]
pub enum WExprCall<FT: IntoSyn<Type>> {
    Call(WCall<FT>),
    MckUnary(WMckUnary),
    MckBinary(WMckBinary),
    MckExt(WMckExt),
    MckNew(WMckNew),
    StdClone(WIdent),
    ArrayRead(WArrayRead),
    ArrayWrite(WArrayWrite),
    Phi(WIdent, WIdent),
    PhiTaken(WIdent),
    PhiNotTaken,
    PhiUninit,
}

#[derive(Clone, Debug, Hash)]
pub enum WHighMckNew {
    Bitvector(u32, i128),
    BitvectorArray(WTypeArray, WIdent),
    Unsigned(u32, i128),
    Signed(u32, i128),
}

#[derive(Clone, Debug, Hash)]
pub enum WMckNew {
    Bitvector(u32, i128),
    BitvectorArray(WTypeArray, WIdent),
}

#[derive(Clone, Debug, Hash)]
pub struct WBitvectorNew {}

#[derive(Clone, Debug, Hash)]
pub struct WArrayNew {
    pub ty: WTypeArray,
    pub fill_element: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WHighMckExt {
    pub width: u32,
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
    pub ty: WHighStdIntoType,
    pub from: WIdent,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WHighStdIntoType {
    Bitvector(u32),
    Unsigned(u32),
    Signed(u32),
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
    pub right: WIdent,
}

pub const MCK_HIGH_EXT: &str = "::machine_check::Ext::ext";
pub const MCK_HIGH_BITVECTOR_NEW: &str = "::machine_check::Bitvector::new";
pub const MCK_HIGH_UNSIGNED_NEW: &str = "::machine_check::Unsigned::new";
pub const MCK_HIGH_SIGNED_NEW: &str = "::machine_check::Signed::new";
pub const MCK_HIGH_BITVECTOR_ARRAY_NEW: &str = "::machine_check::BitvectorArray::new_filled";

pub const MCK_UEXT: &str = "::mck::forward::Ext::uext";
pub const MCK_SEXT: &str = "::mck::forward::Ext::sext";
pub const MCK_BITVECTOR_NEW: &str = "::mck::concr::Bitvector::new";
pub const MCK_BITVECTOR_ARRAY_NEW: &str = "::mck::concr::Array::new_filled";

pub const STD_CLONE: &str = "::std::clone::Clone::clone";
pub const STD_INTO: &str = "::std::convert::Into::into";

pub const ARRAY_READ: &str = "::mck::forward::ReadWrite::read";
pub const ARRAY_WRITE: &str = "::mck::forward::ReadWrite::write";

pub const PHI: &str = "::mck::forward::PhiArg::phi";
pub const PHI_TAKEN: &str = "::mck::forward::PhiArg::Taken";
pub const PHI_NOT_TAKEN: &str = "::mck::forward::PhiArg::NotTaken";
pub const PHI_UNINIT: &str = "::mck::concr::Phi::uninit";

#[derive(Clone, Debug, Hash)]
pub struct WCall<FT: IntoSyn<Type>> {
    pub fn_path: WPath<FT>,
    pub args: Vec<WCallArg>,
}

#[derive(Clone, Debug, Hash)]
pub enum WCallArg {
    Ident(WIdent),
    Literal(Lit),
}

impl<FT: IntoSyn<Type>> IntoSyn<Expr> for WExprCall<FT> {
    fn into_syn(self) -> Expr {
        // TODO: generics into syn
        let span = Span::call_site();
        let (fn_operand, args, generics) = match self {
            WExprCall::Call(call) => return call.into_syn(),
            WExprCall::MckUnary(call) => {
                let operation = call.op.to_string();
                (operation, vec![WCallArg::Ident(call.operand)], None)
            }
            WExprCall::MckBinary(call) => {
                let operation = call.op.to_string();
                (
                    operation,
                    vec![WCallArg::Ident(call.a), WCallArg::Ident(call.b)],
                    None,
                )
            }
            WExprCall::MckExt(call) => (
                if call.signed {
                    String::from(MCK_SEXT)
                } else {
                    String::from(MCK_UEXT)
                },
                vec![WCallArg::Ident(call.from)],
                Some((2, vec![WGeneric::Const(call.width)])),
            ),
            WExprCall::MckNew(call) => match call {
                WMckNew::BitvectorArray(type_array, ident) => (
                    String::from(MCK_BITVECTOR_ARRAY_NEW),
                    vec![WCallArg::Ident(ident)],
                    Some((
                        1,
                        vec![
                            WGeneric::Const(type_array.index_width),
                            WGeneric::Const(type_array.element_width),
                        ],
                    )),
                ),
                WMckNew::Bitvector(width, constant) => (
                    String::from(MCK_BITVECTOR_NEW),
                    vec![WCallArg::Literal(Lit::Int(LitInt::new(
                        constant.to_string().as_str(),
                        span,
                    )))],
                    Some((1, vec![WGeneric::Const(width)])),
                ),
            },
            WExprCall::StdClone(from) => {
                (String::from(STD_CLONE), vec![WCallArg::Ident(from)], None)
            }
            WExprCall::ArrayRead(read) => (
                String::from(ARRAY_READ),
                vec![WCallArg::Ident(read.base), WCallArg::Ident(read.index)],
                None,
            ),
            WExprCall::ArrayWrite(write) => (
                String::from(ARRAY_WRITE),
                vec![
                    WCallArg::Ident(write.base),
                    WCallArg::Ident(write.index),
                    WCallArg::Ident(write.right),
                ],
                None,
            ),
            WExprCall::Phi(a, b) => (
                String::from(PHI),
                vec![WCallArg::Ident(a), WCallArg::Ident(b)],
                None,
            ),
            WExprCall::PhiTaken(ident) => {
                (String::from(PHI_TAKEN), vec![WCallArg::Ident(ident)], None)
            }
            WExprCall::PhiNotTaken => (String::from(PHI_NOT_TAKEN), vec![], None),
            WExprCall::PhiUninit => (String::from(PHI_UNINIT), vec![], None),
        };
        construct_call(fn_operand, args, generics)
    }
}

impl<FT: IntoSyn<Type>> IntoSyn<Expr> for WExprHighCall<FT> {
    fn into_syn(self) -> Expr {
        // TODO: generics into syn
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
                WHighMckNew::Bitvector(_width, constant) => (
                    String::from(MCK_HIGH_BITVECTOR_NEW),
                    vec![WCallArg::Literal(Lit::Int(LitInt::new(
                        constant.to_string().as_str(),
                        span,
                    )))],
                ),
                WHighMckNew::Unsigned(_width, constant) => (
                    String::from(MCK_HIGH_UNSIGNED_NEW),
                    vec![WCallArg::Literal(Lit::Int(LitInt::new(
                        constant.to_string().as_str(),
                        span,
                    )))],
                ),
                WHighMckNew::Signed(_width, constant) => (
                    String::from(MCK_HIGH_SIGNED_NEW),
                    vec![WCallArg::Literal(Lit::Int(LitInt::new(
                        constant.to_string().as_str(),
                        span,
                    )))],
                ),
            },
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
                    WCallArg::Ident(write.right),
                ],
            ),
            WExprHighCall::Phi(a, b) => (
                String::from(PHI),
                vec![WCallArg::Ident(a), WCallArg::Ident(b)],
            ),
            WExprHighCall::PhiTaken(ident) => {
                (String::from(PHI_TAKEN), vec![WCallArg::Ident(ident)])
            }
            WExprHighCall::PhiNotTaken => (String::from(PHI_NOT_TAKEN), vec![]),
            WExprHighCall::PhiUninit => (String::from(PHI_UNINIT), vec![]),
        };
        construct_call(fn_operand, args, None)
    }
}

impl<FT: IntoSyn<Type>> IntoSyn<Expr> for WCall<FT> {
    fn into_syn(self) -> Expr {
        let path = self.fn_path.into();

        let args = Punctuated::from_iter(self.args.into_iter().map(|arg| match arg {
            WCallArg::Ident(ident) => create_expr_ident(ident.into()),
            WCallArg::Literal(lit) => Expr::Lit(ExprLit {
                attrs: Vec::new(),
                lit,
            }),
        }));

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

fn construct_call(
    fn_operand: String,
    args: Vec<WCallArg>,
    _generics: Option<(usize, Vec<WGeneric<WBasicType>>)>,
) -> Expr {
    let span = Span::call_site();

    // construct the WPath
    let without_leading = fn_operand
        .strip_prefix("::")
        .expect("Special function operand should have a leading prefix");
    let segments: Vec<WPathSegment<WBasicType>> = without_leading
        .split("::")
        .map(|segment| WPathSegment {
            ident: WIdent::new(String::from(segment), span),
            generics: None,
        })
        .collect();

    /*if let Some((generics_position, generics_vec)) = generics {
        segments[generics_position].generics = Some(WGenerics {
            leading_colon: true,
            inner: generics_vec,
        })
    }*/

    WCall {
        fn_path: WPath {
            leading_colon: true,
            segments,
        },
        args,
    }
    .into_syn()
}
