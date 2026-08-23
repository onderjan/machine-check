use machine_check_common::iir::stmt::{IAssignStmt, IIfStmt, IStmt};

use crate::{
    context::WLowContext,
    into_iir::func::WFnData,
    wir::{WStmt, YSsa},
    Error,
};

impl WStmt<YSsa> {
    pub(super) fn into_iir(
        self,
        ctx: &WLowContext,
        fn_data: &WFnData,
    ) -> Result<Option<IStmt>, Error> {
        Ok(match self {
            WStmt::Assign(stmt_assign) => {
                let left_ident = stmt_assign.left.into_iir();
                let left = fn_data
                    .ident_var(&left_ident)
                    .expect("Left-side variable should be in variable map");

                let right = stmt_assign.right.into_iir(ctx, fn_data)?;

                right.map(|right| IStmt::Assign(IAssignStmt { left, right }))
            }
            WStmt::If(stmt_if) => {
                let condition = stmt_if.condition.ident.into_iir();

                let condition = fn_data
                    .ident_var(&condition)
                    .expect("Condition variable should be in variable map");

                let then_block = stmt_if.then_block.into_iir(ctx, fn_data)?;
                let else_block = stmt_if.else_block.into_iir(ctx, fn_data)?;

                Some(IStmt::If(IIfStmt {
                    condition,
                    then_block,
                    else_block,
                }))
            }
        })
    }
}
