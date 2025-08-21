use std::collections::HashMap;

use machine_check_common::iir::{
    expr::{
        call::{IExprCall, IMckBinary, IMckNew},
        IExpr,
    },
    path::IIdent,
    variable::IVarId,
};

use crate::{
    into_iir::FromWirData,
    wir::{WExpr, WExprCall, WIdent, WMckNew},
};

impl WExpr<WExprCall> {
    pub(super) fn into_iir(
        self,
        data: &mut FromWirData,
        ident_var_map: &HashMap<IIdent, IVarId>,
    ) -> IExpr {
        match self {
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
                    let a = from_variable_map(data, mck_binary.a, ident_var_map);
                    let b = from_variable_map(data, mck_binary.b, ident_var_map);
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
    ident: WIdent,
    ident_var_map: &HashMap<IIdent, IVarId>,
) -> IVarId {
    println!("Getting ident {:?} from variable map", ident);
    let ident = ident.into_iir();
    if let Some(global_var_id) = data.global_var_ids.get(&ident) {
        data.used_globals.insert(*global_var_id);
        *global_var_id
    } else if let Some(local_var_id) = ident_var_map.get(&ident) {
        *local_var_id
    } else {
        panic!(
            "Expression variable {:?} should be in local or global variable map",
            ident
        );
    }
}
