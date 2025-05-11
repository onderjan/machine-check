use proc_macro2::Span;
use syn::{token::Brace, Block, Expr, ExprAssign, ExprBlock, ExprIf, Stmt, Token, Type};

use crate::util::{create_expr_path, create_path_from_ident};

use super::{expr::WExpr, path::WIdent, IntoSyn};

#[derive(Clone, Debug, Hash)]
pub struct WBlock<FT: IntoSyn<Type>> {
    pub stmts: Vec<WStmt<FT>>,
}

#[derive(Clone, Debug, Hash)]
pub enum WStmt<FT: IntoSyn<Type>> {
    Assign(WStmtAssign<FT>),
    If(WStmtIf<FT>),
}

#[derive(Clone, Debug, Hash)]
pub struct WStmtAssign<FT: IntoSyn<Type>> {
    pub left_ident: WIdent,
    pub right_expr: WExpr<FT>,
}

#[derive(Clone, Debug, Hash)]
pub struct WStmtIf<FT: IntoSyn<Type>> {
    pub condition: WExpr<FT>,
    pub then_block: WBlock<FT>,
    pub else_block: WBlock<FT>,
}

impl<FT: IntoSyn<Type>> IntoSyn<Block> for WBlock<FT> {
    fn into_syn(self) -> Block {
        let mut stmts = Vec::new();

        for stmt in self.stmts {
            let span = Span::call_site();
            match stmt {
                WStmt::Assign(stmt) => {
                    let right = stmt.right_expr.into_syn();

                    stmts.push(Stmt::Expr(
                        Expr::Assign(ExprAssign {
                            attrs: Vec::new(),
                            left: Box::new(create_expr_path(create_path_from_ident(
                                stmt.left_ident.into(),
                            ))),
                            eq_token: Token![=](span),
                            right: Box::new(right),
                        }),
                        Some(Token![;](span)),
                    ));
                }
                WStmt::If(stmt) => {
                    stmts.push(Stmt::Expr(
                        Expr::If(ExprIf {
                            attrs: Vec::new(),
                            if_token: Token![if](span),
                            cond: Box::new(stmt.condition.into_syn()),
                            then_branch: stmt.then_block.into_syn(),
                            else_branch: Some((
                                Token![else](span),
                                Box::new(Expr::Block(ExprBlock {
                                    attrs: Vec::new(),
                                    label: None,
                                    block: stmt.else_block.into_syn(),
                                })),
                            )),
                        }),
                        Some(Token![;](span)),
                    ));
                }
            }
        }

        Block {
            brace_token: Brace::default(),
            stmts,
        }
    }
}
