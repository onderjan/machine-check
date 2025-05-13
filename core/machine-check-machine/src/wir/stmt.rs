use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, token::Brace, Block, Expr, ExprAssign, ExprBlock, ExprCall, ExprIf,
    ExprLit, Stmt, Token,
};
use syn_path::path;

use crate::util::create_expr_path;

use super::{IntoSyn, WCallArg, ZAssignTypes};

#[derive(Clone, Debug, Hash)]
pub struct WBlock<Z: ZAssignTypes> {
    pub stmts: Vec<WStmt<Z>>,
}

#[derive(Clone, Debug, Hash)]
pub enum WStmt<Z: ZAssignTypes> {
    Assign(WStmtAssign<Z>),
    If(WStmtIf<Z>),
}

#[derive(Clone, Debug, Hash)]
pub struct WStmtAssign<Z: ZAssignTypes> {
    pub left: Z::AssignLeft,
    pub right: Z::AssignRight,
}

#[derive(Clone, Debug, Hash)]
pub struct WStmtIf<Z: ZAssignTypes> {
    pub condition: WCallArg,
    pub then_block: WBlock<Z>,
    pub else_block: WBlock<Z>,
}

impl<Z: ZAssignTypes> IntoSyn<Block> for WBlock<Z> {
    fn into_syn(self) -> Block {
        let mut stmts = Vec::new();

        for stmt in self.stmts {
            let span = Span::call_site();
            match stmt {
                WStmt::Assign(stmt) => {
                    let right = stmt.right.into_syn();

                    stmts.push(Stmt::Expr(
                        Expr::Assign(ExprAssign {
                            attrs: Vec::new(),
                            left: Box::new(stmt.left.into_syn()),
                            eq_token: Token![=](span),
                            right: Box::new(right),
                        }),
                        Some(Token![;](span)),
                    ));
                }
                WStmt::If(stmt) => {
                    // TODO: do not add into_bool
                    let condition = match stmt.condition {
                        WCallArg::Literal(lit) => Expr::Lit(ExprLit { attrs: vec![], lit }),
                        WCallArg::Ident(ident) => Expr::Call(ExprCall {
                            attrs: vec![],
                            func: Box::new(create_expr_path(path!(::mck::concr::Test::into_bool))),
                            paren_token: Default::default(),
                            args: Punctuated::from_iter([ident.into_syn()]),
                        }),
                    };

                    stmts.push(Stmt::Expr(
                        Expr::If(ExprIf {
                            attrs: Vec::new(),
                            if_token: Token![if](span),
                            cond: Box::new(condition),
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
