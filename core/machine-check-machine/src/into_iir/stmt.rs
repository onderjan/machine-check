use std::collections::HashMap;

use machine_check_common::iir::{
    path::IIdent,
    stmt::{IAssignStmt, IStmt},
    variable::IVarId,
};

use crate::{abstr::ZAbstr, into_iir::FromWirData, wir::WStmt};

impl WStmt<ZAbstr> {
    pub(super) fn into_iir(
        self,
        data: &mut FromWirData,
        ident_var_map: &HashMap<IIdent, IVarId>,
    ) -> IStmt {
        match self {
            WStmt::Assign(stmt_assign) => {
                let left_ident = stmt_assign.left.into_iir();
                let left = *ident_var_map
                    .get(&left_ident)
                    .expect("Left-side variable should be in variable map");

                let right = stmt_assign.right.into_iir(data, ident_var_map);

                IStmt::Assign(IAssignStmt { left, right })
            }
            WStmt::If(stmt_if) => {
                todo!()
            }
        }
    }
}
