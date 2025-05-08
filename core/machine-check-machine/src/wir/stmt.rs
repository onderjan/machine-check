use proc_macro2::Span;
use syn::{token::Brace, Block, Expr, ExprAssign, ExprBlock, ExprIf, Stmt, Token};

use crate::util::{create_expr_path, create_path_from_ident};

use super::{expr::WExpr, path::WIdent, IntoSyn, YStage};

#[derive(Clone, Debug, Hash)]
pub struct WBlock {
    pub stmts: Vec<WStmt>,
}

#[derive(Clone, Debug, Hash)]
pub enum WStmt {
    Assign(WStmtAssign),
    If(WStmtIf),
}

#[derive(Clone, Debug, Hash)]
pub struct WStmtAssign {
    pub left_ident: WIdent,
    pub right_expr: WExpr,
}

#[derive(Clone, Debug, Hash)]
pub struct WStmtIf {
    pub condition: WExpr,
    pub then_block: WBlock,
    pub else_block: WBlock,
}

#[derive(Clone, Debug, Hash)]
pub struct WLocal<Y: YStage> {
    pub ident: WIdent,
    pub original: WIdent,
    pub ty: Y::LocalType,
}

impl IntoSyn<Block> for WBlock {
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
