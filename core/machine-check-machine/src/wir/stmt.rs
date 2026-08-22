use proc_macro2::{Literal, Span};
use quote::ToTokens;
use std::fmt::Debug;
use syn::{
    punctuated::Punctuated,
    token::{Brace, Paren},
    Block, Expr, ExprAssign, ExprBlock, ExprCall, ExprIf, Macro, Stmt, StmtMacro, Token, Type,
};
use syn_path::path;

use crate::{
    util::create_expr_path,
    wir::{WTypeId, YStage},
};

use super::{IntoSyn, WIdent, YIfPolarity};

#[derive(Clone, Hash, Debug)]
pub struct WSynBlock(Block);

#[derive(Clone, Hash)]
pub struct WBlock<Y: YStage> {
    pub stmts: Vec<Y::Stmt>,
}

#[derive(Clone, Hash)]
pub enum WMacroableStmt<Y: YStage> {
    Assign(WStmtAssign<Y>),
    If(WStmtIf<Y>),
    PanicMacro(WStmtPanicMacro),
}

#[derive(Clone, Hash)]
pub enum WStmt<Y: YStage> {
    Assign(WStmtAssign<Y>),
    If(WStmtIf<Y>),
}

#[derive(Clone, Hash)]
pub struct WStmtAssign<Y: YStage> {
    pub left: Y::AssignLeft,
    pub right: Y::AssignRight,
}

#[derive(Clone, Hash)]
pub struct WStmtIf<Y: YStage> {
    pub condition: WIfCondition<Y::IfPolarity>,
    pub then_block: WBlock<Y>,
    pub else_block: WBlock<Y>,
}

#[derive(Clone, Debug, Hash)]
pub struct WIfCondition<P: YIfPolarity> {
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

impl IntoSyn<Block> for WSynBlock {
    fn into_syn(self, _type_fn: &impl Fn(WTypeId) -> Type) -> Block {
        self.0
    }
}

impl<Y: YStage> IntoSyn<Block> for WBlock<Y> {
    fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Block {
        let mut stmts = Vec::new();
        for stmt in self.stmts {
            stmts.push(stmt.into_syn(type_fn));
        }

        Block {
            brace_token: Brace::default(),
            stmts,
        }
    }
}

impl<Y: YStage> IntoSyn<Stmt> for WStmt<Y> {
    fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Stmt {
        let span = Span::call_site();
        match self {
            WStmt::Assign(stmt) => {
                let right = stmt.right.into_syn(type_fn);

                Stmt::Expr(
                    Expr::Assign(ExprAssign {
                        attrs: Vec::new(),
                        left: Box::new(stmt.left.into_syn(type_fn)),
                        eq_token: Token![=](span),
                        right: Box::new(right),
                    }),
                    Some(Token![;](span)),
                )
            }
            WStmt::If(stmt) => {
                let condition = {
                    let func_operator = stmt.condition.polarity.into_syn(type_fn);
                    Expr::Call(ExprCall {
                        attrs: vec![],
                        func: Box::new(create_expr_path(func_operator)),
                        paren_token: Default::default(),
                        args: Punctuated::from_iter([stmt.condition.ident.into_syn(type_fn)]),
                    })
                };

                let then_branch = stmt.then_block.into_syn(type_fn);
                let else_branch = (
                    Token![else](span),
                    Box::new(Expr::Block(ExprBlock {
                        attrs: Vec::new(),
                        label: None,
                        block: stmt.else_block.into_syn(type_fn),
                    })),
                );

                Stmt::Expr(
                    Expr::If(ExprIf {
                        attrs: Vec::new(),
                        if_token: Token![if](span),
                        cond: Box::new(condition),
                        then_branch,
                        else_branch: Some(else_branch),
                    }),
                    Some(Token![;](span)),
                )
            }
        }
    }
}

impl<Y: YStage> IntoSyn<Stmt> for WMacroableStmt<Y> {
    fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Stmt {
        let panic_macro = match self {
            WMacroableStmt::Assign(stmt) => return WStmt::Assign(stmt).into_syn(type_fn),
            WMacroableStmt::If(stmt) => return WStmt::If(stmt).into_syn(type_fn),
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

impl<Y: YStage> Debug for WStmt<Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WStmt::Assign(assign) => assign.fmt(f),
            WStmt::If(if_stmt) => if_stmt.fmt(f),
        }
    }
}

impl<Y: YStage> Debug for WMacroableStmt<Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WMacroableStmt::Assign(assign) => assign.fmt(f),
            WMacroableStmt::If(if_stmt) => if_stmt.fmt(f),
            WMacroableStmt::PanicMacro(panic_macro) => panic_macro.fmt(f),
        }
    }
}

impl<Y: YStage> Debug for WStmtAssign<Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = {:?}", self.left, self.right)
    }
}

impl<Y: YStage> Debug for WStmtIf<Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {:?} ", self.condition)?;
        Debug::fmt(&self.then_block, f)?;
        write!(f, " else ")?;
        Debug::fmt(&self.else_block, f)
    }
}

impl<Y: YStage> Debug for WBlock<Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut franz = f.debug_set();

        for stmt in &self.stmts {
            franz.entry(stmt);
        }

        franz.finish()
    }
}
