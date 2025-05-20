use proc_macro2::Span;
use syn::{
    token::{Brace, Bracket},
    Expr, ExprField, ExprIndex, ExprLit, ExprReference, ExprStruct, FieldValue, Lit, Token, Type,
};

use crate::util::create_expr_ident;

use super::{IntoSyn, WIdent, WPath};

#[derive(Clone, Debug, Hash)]
pub enum WExpr<FT: IntoSyn<Type>, CF: IntoSyn<Expr>> {
    Move(WIdent),
    Call(CF),
    Field(WExprField),
    Struct(WExprStruct<FT>),
    Reference(WExprReference),
    Lit(Lit),
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
