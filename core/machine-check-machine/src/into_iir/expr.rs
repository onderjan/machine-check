use std::collections::BTreeMap;

use machine_check_common::iir::{
    expr::{
        call::{IExprCall, IMckNew},
        op::IMckBinary,
        IExpr,
    },
    path::IIdent,
    variable::IVarId,
};

use crate::wir::{WExpr, WExprCall, WIdent, WMckNew};

impl WExpr<WExprCall> {
    // TODO: finish this
    #[allow(unused_variables)]
    pub(super) fn into_iir(self, ident_var_map: &BTreeMap<IIdent, IVarId>) -> Option<IExpr> {
        Some(match self {
            WExpr::Move(ident) => {
                let var_id = *ident_var_map
                    .get(&ident.into_iir())
                    .expect("Left-side variable should be in variable map");
                IExpr::Move(var_id)
            }
            WExpr::Call(expr_call) => IExpr::Call(match expr_call {
                WExprCall::Call(wcall) => todo!(),
                WExprCall::MckUnary(wmck_unary) => todo!(),
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
                WExprCall::StdClone(wident) => todo!(),
                WExprCall::ArrayRead(warray_read) => todo!(),
                WExprCall::ArrayWrite(warray_write) => todo!(),
                WExprCall::Phi(left, right) => {
                    let left = from_variable_map(left, ident_var_map);
                    let right = from_variable_map(right, ident_var_map);
                    IExprCall::Phi(left, right)
                }
                WExprCall::PhiTaken(ident) => {
                    // translate as a move
                    let var_id = *ident_var_map
                        .get(&ident.into_iir())
                        .expect("Left-side variable should be in variable map");
                    return Some(IExpr::Move(var_id));
                }
                WExprCall::PhiMaybeTaken(_) => {
                    panic!("Phi maybe taken should not be here")
                }
                WExprCall::PhiNotTaken => {
                    // do not translate to IIR as it is not needed there
                    return None;
                }
                WExprCall::PhiUninit => panic!("Phi uninit should not be here"),
            }),
            WExpr::Field(wexpr_field) => todo!(),
            WExpr::Struct(wexpr_struct) => todo!(),
            WExpr::Reference(wexpr_reference) => todo!(),
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
