use machine_check_common::ir_common::IrMckBinaryOp;

use crate::{
    context::inferred::lower::create_panic_call,
    wir::{
        WBlock, WExpr, WExprHighCall, WExprLowCall, WIfCondition, WIndexedExpr, WIndexedIdent,
        WMacroableStmt, WMckBinary, WNoIfPolarity, WSpan, WStmt, WStmtAssign, WStmtIf, YLowered,
        YTac,
    },
    Errors,
};

impl super::FnLowerer<'_> {
    pub fn lower_block(&mut self, block: WBlock<YTac>) -> Result<WBlock<YLowered>, Errors> {
        let mut stmts = Vec::new();
        let mut errors = Vec::new();

        for stmt in block.stmts {
            match stmt {
                WMacroableStmt::Assign(stmt) => {
                    let WIndexedIdent::NonIndexed(left) = stmt.left else {
                        todo!("Indexed expr");
                    };
                    let WIndexedExpr::NonIndexed(right) = stmt.right else {
                        todo!("Indexed expr");
                    };
                    match self.lower_expr(right) {
                        Ok(right) => stmts.push(WStmt::Assign(WStmtAssign { left, right })),
                        Err(err) => errors.push(err),
                    }
                }
                WMacroableStmt::If(stmt) => {
                    let then_block = self
                        .lower_block(stmt.then_block)
                        .map_err(|err| errors.push(err));
                    let else_block = self
                        .lower_block(stmt.else_block)
                        .map_err(|err| errors.push(err));

                    if let (Ok(then_block), Ok(else_block)) = (then_block, else_block) {
                        stmts.push(WStmt::If(WStmtIf {
                            condition: stmt.condition,
                            then_block,
                            else_block,
                        }))
                    }
                }
                WMacroableStmt::PanicMacro(_panic_macro) => {
                    // TODO: use the panic macro string

                    let panic_num = self.next_panic_num;
                    self.next_panic_num += 1;

                    stmts.extend(self.replace_panic_if_zero(
                        create_panic_call(panic_num.into()),
                        self.panic_ident.span(),
                    ));
                }
            };
        }

        Errors::errors_vec_to_result(errors)?;

        Ok(WBlock { stmts })
    }

    fn lower_expr(&self, expr: WExpr<WExprHighCall>) -> Result<WExpr<WExprLowCall>, Errors> {
        match expr {
            WExpr::Move(ident) => Ok(WExpr::Move(ident)),
            WExpr::Call(expr_call) => Ok(self.lower_call(expr_call)?),
            WExpr::Field(expr_field) => Ok(WExpr::Field(expr_field)),
            WExpr::Struct(expr_struct) => Ok(WExpr::Struct(expr_struct)),
            WExpr::Reference(expr_reference) => Ok(WExpr::Reference(expr_reference)),
            WExpr::Lit(lit, neg) => Ok(WExpr::Lit(lit, neg)),
        }
    }

    fn replace_panic_if_zero(
        &mut self,
        panic_expr: WExpr<WExprLowCall>,
        span: WSpan,
    ) -> Vec<WStmt<YLowered>> {
        // assign to the panic variable if it is currently zero
        let panic_is_zero_ident = self
            .ident_creator
            .create_temporary_ident(span, self.ctx.boolean_type_id());

        let panic_is_zero_call = WExprLowCall::MckBinary(WMckBinary {
            op: IrMckBinaryOp::Eq,
            a: self.panic_ident.clone(),
            b: self.zero_bitvec_ident.clone(),
        });

        let panic_is_zero_assign = WStmt::Assign(WStmtAssign {
            left: panic_is_zero_ident.clone(),
            right: WExpr::Call(panic_is_zero_call),
        });

        let replace_panic = WStmt::Assign(WStmtAssign {
            left: self.panic_ident.clone(),
            right: panic_expr,
        });

        let replace_panic_if_currently_zero = WStmt::If(WStmtIf {
            condition: WIfCondition {
                polarity: WNoIfPolarity,
                ident: panic_is_zero_ident,
            },
            then_block: WBlock {
                stmts: vec![replace_panic],
            },
            else_block: WBlock { stmts: vec![] },
        });

        vec![panic_is_zero_assign, replace_panic_if_currently_zero]
    }
}
