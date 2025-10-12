use std::collections::BTreeMap;

use machine_check_common::iir::{
    path::IIdent,
    stmt::{IAssignStmt, IIfStmt, IStmt},
    variable::IVarId,
};

use crate::{abstr::ZAbstr, wir::WStmt};

impl WStmt<ZAbstr> {
    pub(super) fn into_iir(self, ident_var_map: &BTreeMap<IIdent, IVarId>) -> IStmt {
        match self {
            WStmt::Assign(stmt_assign) => {
                let left_ident = stmt_assign.left.into_iir();
                let left = *ident_var_map
                    .get(&left_ident)
                    .expect("Left-side variable should be in variable map");

                let right = stmt_assign.right.into_iir(ident_var_map);

                IStmt::Assign(IAssignStmt { left, right })
            }
            // TODO: finish this
            #[allow(unused_variables)]
            WStmt::If(stmt_if) => {
                let (condition, is_positive) = match stmt_if.condition {
                    crate::wir::WIfCondition::Ident(condition_ident) => {
                        let is_positive = condition_ident.polarity.0;
                        (condition_ident.ident.into_iir(), is_positive)
                    }
                    crate::wir::WIfCondition::Literal(lit) => todo!("Literals in conditions"),
                };

                let condition = *ident_var_map
                    .get(&condition)
                    .expect("Condition variable should be in variable map");

                let then_block = stmt_if.then_block.into_iir(ident_var_map);
                let else_block = stmt_if.else_block.into_iir(ident_var_map);

                IStmt::If(IIfStmt {
                    condition,
                    is_positive,
                    then_block,
                    else_block,
                })
            }
        }
    }
}
