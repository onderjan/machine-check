use syn::{Block, Expr, ExprBlock, Stmt};

use crate::{
    util::extract_expr_ident,
    wir::{
        WBasicType, WBlock, WExpr, WIndexedExpr, WIndexedIdent, WStmt, WStmtAssign, WStmtIf, ZTac,
    },
};

impl super::FunctionFolder<'_> {
    pub fn fold_block(&mut self, block: Block) -> (WBlock<ZTac>, Option<WExpr<WBasicType>>) {
        let mut orig_stmts = block.stmts;

        let result_stmt = if let Some(Stmt::Expr(_, None)) = orig_stmts.last() {
            orig_stmts.pop()
        } else {
            None
        };

        let mut stmts = Vec::new();

        for orig_stmt in orig_stmts {
            match orig_stmt {
                Stmt::Local(_) => {
                    // do not process here
                }
                Stmt::Expr(stmt_expr, semi) => {
                    assert!(semi.is_some());
                    self.fold_stmt_expr(stmt_expr, &mut stmts);
                }
                _ => panic!("Unexpected type of statement: {:?}", orig_stmt),
            };
        }

        let return_ident: Option<WExpr<WBasicType>> =
            // has a return statement
            if let Some(result_stmt) = result_stmt {
                let Stmt::Expr(expr, None) = result_stmt else {
                    panic!(
                        "Result statement should be an expression: {:?}",
                        result_stmt
                    );
                };
                let WIndexedExpr::NonIndexed(last_expr) = self.fold_right_expr(expr, &mut stmts) else {
                    panic!("Indexed result expression not supported");
                };
                Some(last_expr)
        } else {
            None
        };

        (WBlock { stmts }, return_ident)
    }

    fn fold_stmt_expr(&mut self, stmt_expr: Expr, result_stmts: &mut Vec<WStmt<ZTac>>) {
        match stmt_expr {
            syn::Expr::Assign(expr) => {
                let left = match *expr.left {
                    Expr::Index(expr_index) => {
                        let Some(base_ident) = extract_expr_ident(&expr_index.expr).cloned() else {
                            println!("Left expr: {}", quote::quote! {#expr_index});
                            panic!("Left expr base should be ident");
                        };
                        let index_ident =
                            self.force_right_expr_to_ident(*expr_index.index, result_stmts);
                        WIndexedIdent::Indexed(base_ident.into(), index_ident)
                    }
                    Expr::Path(_) => {
                        let Some(left_ident) = extract_expr_ident(&expr.left) else {
                            panic!("Assignment left should be ident");
                        };
                        WIndexedIdent::NonIndexed(left_ident.clone().into())
                    }
                    _ => panic!("Left expr should be ident or index"),
                };

                let right = self.fold_right_expr(*expr.right, result_stmts);

                // TODO indexed
                result_stmts.push(WStmt::Assign(WStmtAssign { left, right }));
            }
            syn::Expr::If(expr_if) => {
                let Expr::Block(ExprBlock {
                    block: else_block, ..
                }) = *expr_if
                    .else_branch
                    .expect("Else branch should be present")
                    .1
                else {
                    panic!("Else should have a block");
                };

                let condition = self.force_right_expr_to_call_arg(*expr_if.cond, result_stmts);

                result_stmts.push(WStmt::If(WStmtIf {
                    condition,
                    then_block: self.fold_block(expr_if.then_branch).0,
                    else_block: self.fold_block(else_block).0,
                }));
            }
            syn::Expr::Block(expr_block) => {
                // TODO: there should not be nested blocks here
                let (mut block, result) = self.fold_block(expr_block.block);
                assert!(result.is_none());
                result_stmts.append(&mut block.stmts);
            }
            _ => panic!("Unexpected type of expression: {:?}", stmt_expr),
        };
    }
}
