use crate::{
    into_wir::{conversion::lower::lower_basic_path, Errors},
    wir::{
        WBlock, WExpr, WExprHighCall, WExprLowCall, WExprStruct, WIndexedExpr, WIndexedIdent,
        WMacroableStmt, WStmt, WStmtAssign, WStmtIf, ZLowered, ZTac,
    },
};

impl super::FnLowerer<'_> {
    pub fn lower_block(&self, block: WBlock<ZTac>) -> Result<WBlock<ZLowered>, Errors> {
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
                WMacroableStmt::PanicMacro(panic_macro) => todo!("Lower panic macro"),
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
            WExpr::Struct(expr_struct) => Ok(WExpr::Struct(WExprStruct {
                type_path: lower_basic_path(expr_struct.type_path),
                fields: expr_struct.fields,
            })),
            WExpr::Reference(expr_reference) => Ok(WExpr::Reference(expr_reference)),
            WExpr::Lit(lit, neg) => Ok(WExpr::Lit(lit, neg)),
        }
    }
}
