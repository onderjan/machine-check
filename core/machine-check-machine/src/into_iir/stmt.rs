use machine_check_common::iir::stmt::{IAssignStmt, IIfStmt, IStmt};

use crate::{
    into_iir::func::WFnData,
    wir::{WStmt, ZConverted},
};

impl WStmt<ZConverted> {
    pub(super) fn into_iir(self, fn_data: &WFnData) -> Option<IStmt> {
        match self {
            WStmt::Assign(stmt_assign) => {
                let left_ident = stmt_assign.left.into_iir();
                let left = fn_data
                    .ident_var(&left_ident)
                    .expect("Left-side variable should be in variable map");

                stmt_assign
                    .right
                    .into_iir(fn_data)
                    .map(|right| IStmt::Assign(IAssignStmt { left, right }))
            }
            WStmt::If(stmt_if) => {
                let condition = stmt_if.condition.ident.into_iir();

                let condition = fn_data
                    .ident_var(&condition)
                    .expect("Condition variable should be in variable map");

                let then_block = stmt_if.then_block.into_iir(fn_data);
                let else_block = stmt_if.else_block.into_iir(fn_data);

                Some(IStmt::If(IIfStmt {
                    condition,
                    then_block,
                    else_block,
                }))
            }
        }
    }
}
