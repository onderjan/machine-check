use std::fmt::Debug;
use syn::{Expr, Type};

use crate::wir::{call::construct_call_fn_path, WCall, WCallArg, WTypeId};

use super::{IntoSyn, WStdBinary, WStdUnary};

#[derive(Clone, Debug, Hash)]
pub enum WExprHighCall {
    Call(WCall),
    StdUnary(WStdUnary),
    StdBinary(WStdBinary),
}

pub const MCK_HIGH_EXT: &str = "::machine_check::Ext::ext";
pub const MCK_HIGH_BITVECTOR_NEW: &str = "::machine_check::Bitvector::new";
pub const MCK_HIGH_UNSIGNED_NEW: &str = "::machine_check::Unsigned::new";
pub const MCK_HIGH_SIGNED_NEW: &str = "::machine_check::Signed::new";
pub const MCK_HIGH_BITVECTOR_ARRAY_NEW: &str = "::machine_check::BitvectorArray::new_filled";

impl IntoSyn<Expr> for WExprHighCall {
    fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Expr {
        let (fn_operand, args) = match self {
            WExprHighCall::Call(call) => return call.into_syn(type_fn),
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
        WCall { fn_path, args }.into_syn(type_fn)
    }
}
