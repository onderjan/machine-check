use std::collections::BTreeMap;

use machine_check_common::iir::{
    expr::{
        call::{IArrayRead, IExprCall, IMckNew, IPhiTaken},
        op::{IMckBinary, IMckUnary},
        IExpr, IExprReference,
    },
    path::IIdent,
    variable::IVarId,
};

use crate::wir::{WExpr, WExprCall, WExprReference, WIdent, WMckNew};

impl WExpr<WExprCall> {
    // TODO: finish this
    #[allow(unused_variables)]
    pub(super) fn into_iir(self, ident_var_map: &BTreeMap<IIdent, IVarId>) -> Option<IExpr> {
        Some(match self {
            WExpr::Move(ident) => {
                let var_id = from_variable_map(ident, ident_var_map);
                IExpr::Move(var_id)
            }
            WExpr::Call(expr_call) => IExpr::Call(match expr_call {
                WExprCall::Call(wcall) => todo!(),
                WExprCall::MckUnary(mck_unary) => {
                    let operand = from_variable_map(mck_unary.operand, ident_var_map);
                    IExprCall::MckUnary(IMckUnary {
                        op: mck_unary.op,
                        operand,
                    })
                }
                WExprCall::MckBinary(mck_binary) => {
                    let a = from_variable_map(mck_binary.a, ident_var_map);
                    let b = from_variable_map(mck_binary.b, ident_var_map);
                    IExprCall::MckBinary(IMckBinary {
                        op: mck_binary.op,
                        a,
                        b,
                    })
                }
                WExprCall::MckExt(wmck_ext) => todo!(),
                WExprCall::MckNew(mck_new) => IExprCall::MckNew(match mck_new {
                    WMckNew::Bitvector(width, constant) => IMckNew::Bitvector(width, constant),
                    WMckNew::BitvectorArray(wtype_array, wident) => todo!(),
                }),
                WExprCall::BooleanNew(value) => IExprCall::BooleanNew(value),
                WExprCall::StdClone(wident) => todo!(),
                WExprCall::ArrayRead(array_read) => IExprCall::ArrayRead(IArrayRead {
                    base: from_variable_map(array_read.base, ident_var_map),
                    index: from_variable_map(array_read.index, ident_var_map),
                }),
                WExprCall::ArrayWrite(warray_write) => todo!(),
                WExprCall::Phi(left, right) => {
                    let left = from_variable_map(left, ident_var_map);
                    let right = from_variable_map(right, ident_var_map);
                    IExprCall::Phi(left, right)
                }
                WExprCall::PhiTaken(taken) => {
                    // translate as a move
                    let var = from_variable_map(taken.ident, ident_var_map);
                    let condition = from_variable_map(taken.condition, ident_var_map);
                    IExprCall::PhiTaken(IPhiTaken { var, condition })
                }
                WExprCall::PhiNotTaken => {
                    // do not translate to IIR as it is not needed there
                    return None;
                }
                WExprCall::PhiUninit => panic!("Phi uninit should not be here"),
            }),
            WExpr::Field(wexpr_field) => todo!(),
            WExpr::Struct(wexpr_struct) => todo!(),
            WExpr::Reference(expr_reference) => IExpr::Reference(match expr_reference {
                WExprReference::Ident(ident) => {
                    IExprReference::Ident(from_variable_map(ident, ident_var_map))
                }
                WExprReference::Field(expr_field) => todo!(),
            }),
            WExpr::Lit(lit) => todo!(),
        })
    }
}

fn from_variable_map(ident: WIdent, ident_var_map: &BTreeMap<IIdent, IVarId>) -> IVarId {
    let ident = ident.into_iir();
    if let Some(local_var_id) = ident_var_map.get(&ident) {
        *local_var_id
    } else {
        panic!(
            "Expression variable {:?} should be in local or global variable map",
            ident
        );
    }
}
