use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    punctuated::Punctuated,
    token::{Brace, Bracket, Paren},
    Expr, ExprCall, ExprField, ExprIndex, ExprLit, ExprPath, ExprReference, ExprStruct, FieldValue,
    Lit, Path, Token, Type,
};

use crate::util::create_expr_ident;

use super::{IntoSyn, WIdent, WPath};

#[derive(Clone, Debug, Hash)]
pub enum WExpr<FT: IntoSyn<Type>, CF: IntoSyn<Expr>> {
    Move(WIdent),
    Call(WExprCall<CF>),
    Field(WExprField),
    Struct(WExprStruct<FT>),
    Reference(WExprReference),
    Lit(Lit),
}

#[derive(Clone, Debug, Hash)]
pub struct WExprCall<CF: IntoSyn<Expr>> {
    pub fn_path: CF,
    pub args: Vec<WCallArg>,
}

#[derive(Clone, Debug, Hash)]
pub enum WHighLevelCallFunc<FT: IntoSyn<Type>> {
    Call(WPath<FT>),
}

#[derive(Clone, Debug, Hash)]
pub struct WCallFunc<FT: IntoSyn<Type>>(pub WPath<FT>);

impl<CF: IntoSyn<Expr>> WExprCall<CF> {
    pub fn span(&self) -> Span {
        // TODO: correct span
        Span::call_site()
    }
}

#[derive(Clone, Debug, Hash)]
pub enum WCallArg {
    Ident(WIdent),
    Literal(Lit),
}

#[derive(Clone, Debug, Hash)]
pub struct WExprField {
    pub base: WIdent,
    pub member: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WExprStruct<FT: IntoSyn<Type>> {
    pub type_path: WPath<FT>,
    pub fields: Vec<(WIdent, WIdent)>,
}

#[derive(Clone, Debug, Hash)]
pub enum WExprReference {
    Ident(WIdent),
    Field(WExprField),
}

#[derive(Clone, Debug, Hash)]
pub enum WIndexedExpr<FT: IntoSyn<Type>, CF: IntoSyn<Expr>> {
    Indexed(WArrayBaseExpr, WIdent),
    NonIndexed(WExpr<FT, CF>),
}

#[derive(Clone, Debug, Hash)]
pub enum WArrayBaseExpr {
    Ident(WIdent),
    Field(WExprField),
}

#[derive(Clone, Debug, Hash)]
pub enum WIndexedIdent {
    Indexed(WIdent, WIdent),
    NonIndexed(WIdent),
}

impl<FT: IntoSyn<Type>, CF: IntoSyn<Expr>> IntoSyn<Expr> for WExpr<FT, CF> {
    fn into_syn(self) -> Expr {
        let span = Span::call_site();
        match self {
            WExpr::Move(ident) => create_expr_ident(ident.into()),
            WExpr::Call(expr) => expr.into_syn(),
            WExpr::Field(expr) => Expr::Field(ExprField {
                attrs: Vec::new(),
                base: Box::new(create_expr_ident(expr.base.into())),
                dot_token: Token![.](span),
                member: syn::Member::Named(expr.member.into()),
            }),
            WExpr::Struct(expr) => {
                let fields = expr
                    .fields
                    .into_iter()
                    .map(|(name, value)| FieldValue {
                        attrs: Vec::new(),
                        member: syn::Member::Named(name.into()),
                        colon_token: Some(Token![:](span)),
                        expr: create_expr_ident(value.into()),
                    })
                    .collect();

                Expr::Struct(ExprStruct {
                    attrs: Vec::new(),
                    qself: None,
                    path: expr.type_path.into(),
                    brace_token: Brace::default(),
                    fields,
                    dot2_token: None,
                    rest: None,
                })
            }
            WExpr::Reference(expr) => {
                let inner = match expr {
                    WExprReference::Ident(ident) => create_expr_ident(ident.into()),
                    WExprReference::Field(expr) => Expr::Field(ExprField {
                        attrs: Vec::new(),
                        base: Box::new(create_expr_ident(expr.base.into())),
                        dot_token: Token![.](span),
                        member: syn::Member::Named(expr.member.into()),
                    }),
                };
                Expr::Reference(ExprReference {
                    attrs: Vec::new(),
                    and_token: Token![&](span),
                    mutability: None,
                    expr: Box::new(inner),
                })
            }
            WExpr::Lit(lit) => Expr::Lit(ExprLit {
                attrs: Vec::new(),
                lit,
            }),
        }
    }
}

impl<FT: IntoSyn<Type>, CF: IntoSyn<Expr>> IntoSyn<Expr> for WIndexedExpr<FT, CF> {
    fn into_syn(self) -> Expr {
        match self {
            WIndexedExpr::Indexed(array, index) => {
                let array = match array {
                    WArrayBaseExpr::Ident(ident) => ident.into_syn(),
                    WArrayBaseExpr::Field(field) => Expr::Field(ExprField {
                        attrs: Vec::new(),
                        base: Box::new(field.base.into_syn()),
                        dot_token: Token![.](index.span()),
                        member: syn::Member::Named(field.member.into()),
                    }),
                };
                indexed_ident(array, index.into_syn())
            }

            WIndexedExpr::NonIndexed(expr) => expr.into_syn(),
        }
    }
}

impl<CF: IntoSyn<Expr>> IntoSyn<Expr> for WExprCall<CF> {
    fn into_syn(self) -> Expr {
        let args = Punctuated::from_iter(self.args.into_iter().map(|arg| match arg {
            WCallArg::Ident(ident) => create_expr_ident(ident.into()),
            WCallArg::Literal(lit) => Expr::Lit(ExprLit {
                attrs: Vec::new(),
                lit,
            }),
        }));
        match self.fn_path.into_syn() {
            Expr::Call(mut expr_call) => {
                assert!(expr_call.args.is_empty());
                expr_call.args = args;
                Expr::Call(expr_call)
            }
            Expr::Macro(mut expr_macro) => {
                assert!(expr_macro.mac.tokens.is_empty());
                let token_stream = args.to_token_stream();
                expr_macro.mac.tokens = token_stream;
                Expr::Macro(expr_macro)
            }
            _ => panic!("Unexpected expr type when converting expr call to syn"),
        }
    }
}

impl<FT: IntoSyn<Type>> IntoSyn<Expr> for WCallFunc<FT> {
    fn into_syn(self) -> Expr {
        create_expr_call_without_args(self.0.into())
    }
}

impl<FT: IntoSyn<Type>> IntoSyn<Expr> for WHighLevelCallFunc<FT> {
    fn into_syn(self) -> Expr {
        match self {
            WHighLevelCallFunc::Call(path) => create_expr_call_without_args(path.into()),
        }
    }
}

impl IntoSyn<Expr> for WIndexedIdent {
    fn into_syn(self) -> Expr {
        match self {
            WIndexedIdent::Indexed(array, index) => {
                indexed_ident(array.into_syn(), index.into_syn())
            }
            WIndexedIdent::NonIndexed(ident) => ident.into_syn(),
        }
    }
}

fn indexed_ident(array: Expr, index: Expr) -> Expr {
    Expr::Index(ExprIndex {
        attrs: Vec::new(),
        expr: Box::new(array),
        bracket_token: Bracket::default(),
        index: Box::new(index),
    })
}

fn create_expr_call_without_args(path: Path) -> Expr {
    Expr::Call(ExprCall {
        attrs: Vec::new(),
        func: Box::new(Expr::Path(ExprPath {
            attrs: vec![],
            path,
            qself: None,
        })),
        paren_token: Paren::default(),
        args: Punctuated::default(),
    })
}
