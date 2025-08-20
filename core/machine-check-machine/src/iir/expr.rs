mod call;

use std::collections::HashMap;

use crate::{
    iir::{
        expr::call::{IExprCall, IMckBinary, IMckNew},
        interpretation::{IAbstractValue, IRefinementValue, Interpretation},
        variable::IVarId,
        FromWirData,
    },
    wir::{WExpr, WExprCall, WIdent, WMckNew},
};

#[derive(Clone, Debug, Hash)]
pub enum IExpr {
    Move(IVarId),
    Call(IExprCall),
    /*Field(IExprField),
    Struct(IExprStruct),
    Reference(IExprReference),
    Lit(Lit),*/
}

impl IExpr {
    pub fn forward_interpret(&self, inter: &mut Interpretation) -> IAbstractValue {
        match self {
            IExpr::Move(var_id) => inter.abstract_value(*var_id).clone(),
            IExpr::Call(expr_call) => expr_call.forward_interpret(inter),
        }
    }

    pub fn backward_interpret(&self, inter: &mut Interpretation, later: IRefinementValue) {
        match self {
            IExpr::Move(var_id) => {
                // propagate the later value to earlier
                inter.insert_refinement_value(*var_id, later);
            }
            IExpr::Call(expr_call) => expr_call.backward_interpret(inter, later),
        }
    }

    pub(super) fn from_wir(
        data: &mut FromWirData,
        expr: WExpr<WExprCall>,
        ident_var_map: &HashMap<WIdent, IVarId>,
    ) -> Self {
        match expr {
            WExpr::Move(ident) => {
                let var_id = *ident_var_map
                    .get(&ident)
                    .expect("Left-side variable should be in variable map");
                IExpr::Move(var_id)
            }
            WExpr::Call(expr_call) => IExpr::Call(match expr_call {
                WExprCall::Call(wcall) => todo!(),
                WExprCall::MckUnary(wmck_unary) => todo!(),
                WExprCall::MckBinary(mck_binary) => {
                    let a = from_variable_map(data, &mck_binary.a, ident_var_map);
                    let b = from_variable_map(data, &mck_binary.b, ident_var_map);
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
                WExprCall::Phi(wident, wident1) => todo!(),
                WExprCall::PhiTaken(wident) => todo!(),
                WExprCall::PhiMaybeTaken(wphi_maybe_taken) => todo!(),
                WExprCall::PhiNotTaken => todo!(),
                WExprCall::PhiUninit => todo!(),
            }),
            WExpr::Field(wexpr_field) => todo!(),
            WExpr::Struct(wexpr_struct) => todo!(),
            WExpr::Reference(wexpr_reference) => todo!(),
            WExpr::Lit(lit) => todo!(),
        }
    }
}

fn from_variable_map(
    data: &mut FromWirData,
    ident: &WIdent,
    ident_var_map: &HashMap<WIdent, IVarId>,
) -> IVarId {
    if let Some(local_var_id) = ident_var_map.get(ident) {
        *local_var_id
    } else if let Some((global_var_id, _)) = data.global_vars.get(ident) {
        data.used_globals.insert(*global_var_id);
        *global_var_id
    } else {
        panic!(
            "Expression variable {:?} should be in local or global variable map",
            ident
        );
    }
}
