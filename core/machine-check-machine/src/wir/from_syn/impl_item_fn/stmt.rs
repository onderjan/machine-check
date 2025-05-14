use std::collections::HashMap;

use syn::{Block, Expr, ExprBlock, Pat, Stmt};

use crate::wir::{
    from_syn::{impl_item_fn::FunctionScope, ty::fold_partial_general_type},
    WBasicType, WBlock, WExpr, WIdent, WIndexedIdent, WPartialGeneralType, WStmt, WStmtAssign,
    WStmtIf, ZTac,
};

impl super::FunctionFolder {
    pub fn fold_block(&mut self, block: Block) -> (WBlock<ZTac>, Option<WExpr<WBasicType>>) {
        // push a local scope
        let scope_id = self.next_scope_id;
        self.next_scope_id = self
            .next_scope_id
            .checked_add(1)
            .expect("Scope id should not overflow");
        self.scopes.push(FunctionScope {
            local_map: HashMap::new(),
        });

        let mut orig_stmts = block.stmts;

        let result_stmt = if let Some(Stmt::Expr(_, None)) = orig_stmts.last() {
            orig_stmts.pop()
        } else {
            None
        };

        let mut stmts = Vec::new();

        for orig_stmt in orig_stmts {
            match orig_stmt {
                Stmt::Local(local) => {
                    let mut pat = local.pat.clone();
                    let mut ty = WPartialGeneralType::Unknown;
                    if let Pat::Type(pat_type) = pat {
                        ty = fold_partial_general_type(*pat_type.ty);
                        pat = *pat_type.pat;
                    }

                    let Pat::Ident(left_pat_ident) = pat else {
                        // TODO: this should be an error
                        panic!("Local pattern should be an ident: {:?}", pat)
                    };
                    let original_ident = WIdent::from_syn_ident(left_pat_ident.ident);
                    self.add_local_ident(scope_id, original_ident, ty);
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
                let ident= self.force_right_expr_to_ident(expr, &mut stmts);
                Some(WExpr::Move(ident))
        } else {
            None
        };

        // pop the local scope, it should exist
        assert!(self.scopes.pop().is_some());

        (WBlock { stmts }, return_ident)
    }

    fn fold_stmt_expr(&mut self, stmt_expr: Expr, result_stmts: &mut Vec<WStmt<ZTac>>) {
        match stmt_expr {
            syn::Expr::Assign(expr) => {
                let left = match *expr.left {
                    Expr::Index(expr_index) => {
                        let base_ident = self.fold_expr_as_ident(*expr_index.expr);

                        let index_ident =
                            self.force_right_expr_to_ident(*expr_index.index, result_stmts);
                        WIndexedIdent::Indexed(base_ident, index_ident)
                    }
                    Expr::Path(expr_path) => {
                        let left_ident = self.fold_expr_as_ident(Expr::Path(expr_path));

                        WIndexedIdent::NonIndexed(left_ident.clone())
                    }
                    _ => panic!("Left expr should be ident or index"),
                };

                let right = self.fold_right_expr(*expr.right, result_stmts);

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
                // handle nested blocks
                let (mut block, result) = self.fold_block(expr_block.block);
                assert!(result.is_none());
                result_stmts.append(&mut block.stmts);
            }
            _ => panic!("Unexpected type of expression: {:?}", stmt_expr),
        };
    }
}
