use std::collections::BTreeMap;

use machine_check_common::iir::{
    path::IIdent,
    stmt::{IAssignStmt, IIfStmt, IStmt},
    variable::IVarId,
};

use crate::wir::{WStmt, ZConverted};

impl WStmt<ZConverted> {
    pub(super) fn into_iir(self, ident_var_map: &BTreeMap<IIdent, IVarId>) -> Option<IStmt> {
        match self {
            WStmt::Assign(stmt_assign) => {
                let left_ident = stmt_assign.left.into_iir();
                let left = *ident_var_map
                    .get(&left_ident)
                    .expect("Left-side variable should be in variable map");

                stmt_assign
                    .right
                    .into_iir(ident_var_map)
                    .map(|right| IStmt::Assign(IAssignStmt { left, right }))
            }
            // TODO: finish this
            #[allow(unused_variables)]
            WStmt::If(stmt_if) => {
                let condition = stmt_if.condition.ident.into_iir();

                let condition = *ident_var_map
                    .get(&condition)
                    .expect("Condition variable should be in variable map");

                let then_block = stmt_if.then_block.into_iir(ident_var_map);
                let else_block = stmt_if.else_block.into_iir(ident_var_map);

                Some(IStmt::If(IIfStmt {
                    condition,
                    then_block,
                    else_block,
                }))
            }
        }
    }
}
