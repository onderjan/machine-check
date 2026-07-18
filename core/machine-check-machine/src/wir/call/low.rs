use machine_check_common::{ir_common::IrTypeArray, Signedness};
use std::fmt::Debug;
use syn::{punctuated::Punctuated, token::Paren, Expr, ExprCall, ExprPath};

use crate::wir::IntoSyn;

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
    fn into_syn(self) -> Expr {
        let (func_path, args) = match self {
            WExprLowCall::Call(call) => {
                return call.into_syn();
                /*let func = call.fn_path.into();
                let args = Punctuated::from_iter(call.args.iter().map(|arg| arg.into() ));
                (func, args)*/
            }
            WExprLowCall::MckUnary(unary) => todo!("unary"),
            WExprLowCall::MckBinary(binary) => todo!("binary"),
            WExprLowCall::MckExt(ext) => todo!("ext"),
            WExprLowCall::MckNew(new) => todo!("new"),
            WExprLowCall::BooleanNew(value) => todo!("value"),
            WExprLowCall::StdClone(ident) => todo!("ident"),
            WExprLowCall::ArrayRead(array_read) => todo!("array read"),
            WExprLowCall::ArrayWrite(array_write) => todo!("array write"),
            WExprLowCall::Phi(phi) => todo!("phi"),
            WExprLowCall::PhiTaken(phi_taken) => todo!("phi taken"),
            WExprLowCall::PhiNotTaken => todo!("phi not taken"),
        };

        Expr::Call(ExprCall {
            attrs: Vec::new(),
            func: Box::new(Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: func_path,
            })),
            paren_token: Paren::default(),
            args,
        })
    }
}
