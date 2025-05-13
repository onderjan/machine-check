use syn::{Block, Expr, ExprBlock, Stmt};

use crate::{
    util::extract_expr_ident,
    wir::{
        WBasicType, WBlock, WExpr, WIndexedExpr, WIndexedIdent, WStmt, WStmtAssign, WStmtIf, ZTac,
    },
};

use super::expr::fold_right_expr;

pub fn fold_block(block: Block) -> (WBlock<ZTac>, Option<WExpr<WBasicType>>) {
    let mut orig_stmts = block.stmts;
    let return_ident: Option<WExpr<WBasicType>> =
        if let Some(Stmt::Expr(_, None)) = orig_stmts.last() {
            // has a return statement
            orig_stmts.pop().map(|stmt| {
                let Stmt::Expr(expr, None) = stmt else {
                    panic!("Return statement should be an expression: {:?}", stmt);
                };
                let WIndexedExpr::NonIndexed(expr) = fold_right_expr(expr) else {
                    panic!("Indexed return statement not supported");
                };
                expr
            })
        } else {
            None
        };

    let mut stmts = Vec::new();

    for orig_stmt in orig_stmts {
        match orig_stmt {
            Stmt::Local(_) => {
                // do not process here
            }
            Stmt::Expr(expr, semi) => {
                assert!(semi.is_some());
                match expr {
                    syn::Expr::Assign(expr) => {
                        let left = match *expr.left {
                            Expr::Index(expr_index) => {
                                let Some(base_ident) =
                                    extract_expr_ident(&expr_index.expr).cloned()
                                else {
                                    println!("Left expr: {}", quote::quote! {#expr_index});
                                    panic!("Left expr base should be ident");
                                };
                                let Some(index_ident) =
                                    extract_expr_ident(&expr_index.index).cloned()
                                else {
                                    println!("Left expr: {}", quote::quote! {#expr_index});
                                    panic!("Left expr index should be ident");
                                };
                                WIndexedIdent::Indexed(base_ident.into(), index_ident.into())
                            }
                            Expr::Path(_) => {
                                let Some(left_ident) = extract_expr_ident(&expr.left) else {
                                    panic!("Assignment left should be ident");
                                };
                                WIndexedIdent::NonIndexed(left_ident.clone().into())
                            }
                            _ => panic!("Left expr should be ident or index"),
                        };

                        // TODO indexed
                        stmts.push(WStmt::Assign(WStmtAssign {
                            left,
                            right: fold_right_expr(*expr.right),
                        }));
                    }
                    syn::Expr::If(expr_if) => {
                        let Expr::Block(ExprBlock {
                            block: else_block, ..
                        }) = *expr_if.else_branch.unwrap().1
                        else {
                            panic!("Else should have a block");
                        };

                        let WIndexedExpr::NonIndexed(condition) = fold_right_expr(*expr_if.cond)
                        else {
                            panic!("Indexed expressions in conditions not supported");
                        };

                        stmts.push(WStmt::If(WStmtIf {
                            condition,
                            then_block: fold_block(expr_if.then_branch).0,
                            else_block: fold_block(else_block).0,
                        }));
                    }
                    syn::Expr::Block(expr_block) => {
                        // TODO: there should not be nested blocks here
                        let (mut block, result) = fold_block(expr_block.block);
                        assert!(result.is_none());
                        stmts.append(&mut block.stmts);
                    }
                    _ => panic!("Unexpected type of expression: {:?}", expr),
                };
            }
            _ => panic!("Unexpected type of statement: {:?}", orig_stmt),
        };
    }

    (WBlock { stmts }, return_ident)
}
