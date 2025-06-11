use proc_macro2::{Literal, Span};
use quote::ToTokens;
use syn::{
    punctuated::Punctuated,
    token::{Brace, Paren},
    Block, Expr, ExprAssign, ExprBlock, ExprCall, ExprIf, ExprLit, Lit, Macro, Stmt, StmtMacro,
    Token,
};
use syn_path::path;

use crate::util::create_expr_path;

use super::{IntoSyn, WIdent, ZAssignTypes, ZIfPolarity};

#[derive(Clone, Debug, Hash)]
pub struct WBlock<Z: ZAssignTypes> {
    pub stmts: Vec<Z::Stmt>,
}

#[derive(Clone, Debug, Hash)]
pub enum WMacroableStmt<Z: ZAssignTypes> {
    Assign(WStmtAssign<Z>),
    If(WStmtIf<Z>),
    PanicMacro(WStmtPanicMacro),
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
    pub condition: WIfCondition<Z::IfPolarity>,
    pub then_block: WBlock<Z>,
    pub else_block: WBlock<Z>,
}

#[derive(Clone, Debug, Hash)]
pub enum WIfCondition<P: ZIfPolarity> {
    Ident(WIfConditionIdent<P>),
    Literal(Lit),
}

#[derive(Clone, Debug, Hash)]
pub struct WIfConditionIdent<P: ZIfPolarity> {
    pub polarity: P,
    pub ident: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WStmtPanicMacro {
    pub kind: WPanicMacroKind,
    pub msg: String,
}

#[derive(Clone, Debug, Hash)]
pub enum WPanicMacroKind {
    Panic,
    Unimplemented,
    Todo,
}

impl<Z: ZAssignTypes> IntoSyn<Block> for WBlock<Z> {
    fn into_syn(self) -> Block {
        let stmts = self.stmts.into_iter().map(IntoSyn::into_syn).collect();

        Block {
            brace_token: Brace::default(),
            stmts,
        }
    }
}

impl<Z: ZAssignTypes> IntoSyn<Stmt> for WStmt<Z> {
    fn into_syn(self) -> Stmt {
        let span = Span::call_site();
        match self {
            WStmt::Assign(stmt) => {
                let right = stmt.right.into_syn();

                Stmt::Expr(
                    Expr::Assign(ExprAssign {
                        attrs: Vec::new(),
                        left: Box::new(stmt.left.into_syn()),
                        eq_token: Token![=](span),
                        right: Box::new(right),
                    }),
                    Some(Token![;](span)),
                )
            }
            WStmt::If(stmt) => {
                let condition = match stmt.condition {
                    WIfCondition::Literal(lit) => Expr::Lit(ExprLit { attrs: vec![], lit }),
                    WIfCondition::Ident(condition_ident) => {
                        let func_operator = condition_ident.polarity.into_syn();
                        Expr::Call(ExprCall {
                            attrs: vec![],
                            func: Box::new(create_expr_path(func_operator)),
                            paren_token: Default::default(),
                            args: Punctuated::from_iter([condition_ident.ident.into_syn()]),
                        })
                    }
                };

                Stmt::Expr(
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
                )
            }
        }
    }
}

impl<Z: ZAssignTypes> IntoSyn<Stmt> for WMacroableStmt<Z> {
    fn into_syn(self) -> Stmt {
        let panic_macro = match self {
            WMacroableStmt::Assign(stmt) => return WStmt::Assign(stmt).into_syn(),
            WMacroableStmt::If(stmt) => return WStmt::If(stmt).into_syn(),
            WMacroableStmt::PanicMacro(panic_macro) => panic_macro,
        };
        let span = Span::call_site();

        let path = match panic_macro.kind {
            WPanicMacroKind::Panic => path!(::std::panic),
            WPanicMacroKind::Unimplemented => path!(::std::unimplemented),
            WPanicMacroKind::Todo => path!(::std::todo),
        };

        let mac = Macro {
            path,
            bang_token: Token![!](span),
            delimiter: syn::MacroDelimiter::Paren(Paren::default()),
            tokens: Literal::string(&panic_macro.msg).into_token_stream(),
        };

        Stmt::Macro(StmtMacro {
            attrs: vec![],
            mac,
            semi_token: Some(Token![;](span)),
        })
    }
}
