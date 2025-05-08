use proc_macro2::Span;
use syn::{
    token::{Brace, Bracket},
    Attribute, Block, Expr, ExprAssign, ExprBlock, ExprIf, Local, MetaNameValue, Pat, PatIdent,
    PatType, Stmt, Token,
};
use syn_path::path;

use crate::util::{create_expr_path, create_path_from_ident};

use super::{expr::WExpr, path::WIdent, ty::WType, IntoSyn};

#[derive(Clone, Debug, Hash)]
pub struct WBlock {
    pub locals: Vec<WLocal>,
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
pub struct WLocal {
    pub ident: WIdent,
    pub original: WIdent,
    pub ty: Option<WType>,
}

impl IntoSyn<Block> for WBlock {
    fn into_syn(self) -> Block {
        let mut stmts = Vec::new();

        for local in self.locals {
            let span = local.ident.span;

            let mut pat = Pat::Ident(PatIdent {
                attrs: Vec::new(),
                by_ref: None,
                mutability: None,
                ident: local.ident.into(),
                subpat: None,
            });

            if let Some(ty) = local.ty {
                pat = Pat::Type(PatType {
                    attrs: Vec::new(),
                    pat: Box::new(pat),
                    colon_token: Token![:](span),
                    ty: Box::new(ty.into_syn()),
                });
            }

            stmts.push(syn::Stmt::Local(Local {
                attrs: vec![Attribute {
                    pound_token: Token![#](span),
                    style: syn::AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    meta: syn::Meta::NameValue(MetaNameValue {
                        path: path!(::mck::attr::tmp_original),
                        eq_token: Token![=](span),
                        value: create_expr_path(create_path_from_ident(local.original.into())),
                    }),
                }],
                let_token: Token![let](span),
                pat,
                init: None,
                semi_token: Token![;](span),
            }));
        }

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
