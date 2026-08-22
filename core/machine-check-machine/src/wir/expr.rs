use proc_macro2::Span;
use std::fmt::Debug;
use syn::{
    punctuated::Punctuated,
    token::{Brace, Bracket},
    Expr, ExprField, ExprIndex, ExprLit, ExprReference, ExprStruct, ExprUnary, FieldValue, Index,
    Lit, Token, Type,
};

use crate::{
    util::create_expr_ident,
    wir::{WPartialPath, WTypeId},
};

use super::{IntoSyn, WIdent};

#[derive(Clone, Hash)]
pub enum WExpr<CF: IntoSyn<Expr>> {
    Move(WIdent),
    Call(CF),
    Field(WExprField),
    Struct(WExprStruct),
    Reference(WExprReference),
    Lit(Lit, bool),
}

#[derive(Clone, Debug, Hash)]
pub struct WExprField {
    pub base: WIdent,
    pub member: WIdent,
}

#[derive(Clone, Debug, Hash)]
pub struct WExprStruct {
    pub type_path: WPartialPath,
    pub fields: Vec<(WIdent, WIdent)>,
}

#[derive(Clone, Debug, Hash)]
pub enum WExprReference {
    Ident(WIdent),
    Field(WExprField),
}

#[derive(Clone, Debug, Hash)]
pub enum WIndexedExpr<CF: IntoSyn<Expr>> {
    Indexed(WArrayBaseExpr, WIdent),
    NonIndexed(WExpr<CF>),
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

impl<CF: IntoSyn<Expr>> IntoSyn<Expr> for WExpr<CF> {
    fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Expr {
        let span = Span::call_site();
        match self {
            WExpr::Move(ident) => create_expr_ident(ident.into()),
            WExpr::Call(expr) => expr.into_syn(type_fn),
            WExpr::Field(expr) => Expr::Field(ExprField {
                attrs: Vec::new(),
                base: Box::new(create_expr_ident(expr.base.into())),
                dot_token: Token![.](span),
                member: into_member(expr.member),
            }),
            WExpr::Struct(expr) => {
                let mut fields = Punctuated::new();

                for (name, value) in expr.fields {
                    fields.push(FieldValue {
                        attrs: Vec::new(),
                        member: into_member(name),
                        colon_token: Some(Token![:](span)),
                        expr: create_expr_ident(value.into()),
                    });
                }

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
                        member: into_member(expr.member),
                    }),
                };
                Expr::Reference(ExprReference {
                    attrs: Vec::new(),
                    and_token: Token![&](span),
                    mutability: None,
                    expr: Box::new(inner),
                })
            }
            WExpr::Lit(lit, neg) => {
                let lit_expr = Expr::Lit(ExprLit {
                    attrs: Vec::new(),
                    lit,
                });

                if neg {
                    Expr::Unary(ExprUnary {
                        attrs: Vec::new(),
                        op: syn::UnOp::Neg(Token![-](span)),
                        expr: Box::new(lit_expr),
                    })
                } else {
                    lit_expr
                }
            }
        }
    }
}

fn into_member(member_ident: WIdent) -> syn::Member {
    let Ok(parsed) = member_ident.name().parse() else {
        return syn::Member::Named(member_ident.into());
    };
    syn::Member::Unnamed(Index {
        index: parsed,
        span: member_ident.span(),
    })
}

impl<CF: IntoSyn<Expr>> IntoSyn<Expr> for WIndexedExpr<CF> {
    fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Expr {
        match self {
            WIndexedExpr::Indexed(array, index) => {
                let array = match array {
                    WArrayBaseExpr::Ident(ident) => ident.into_syn(type_fn),
                    WArrayBaseExpr::Field(field) => Expr::Field(ExprField {
                        attrs: Vec::new(),
                        base: Box::new(field.base.into_syn(type_fn)),
                        dot_token: Token![.](index.span()),
                        member: syn::Member::Named(field.member.into()),
                    }),
                };
                indexed_ident(array, index.into_syn(type_fn))
            }

            WIndexedExpr::NonIndexed(expr) => expr.into_syn(type_fn),
        }
    }
}
impl IntoSyn<Expr> for WIndexedIdent {
    fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Expr {
        match self {
            WIndexedIdent::Indexed(array, index) => {
                indexed_ident(array.into_syn(type_fn), index.into_syn(type_fn))
            }
            WIndexedIdent::NonIndexed(ident) => ident.into_syn(type_fn),
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

impl<CF: IntoSyn<Expr> + Debug> Debug for WExpr<CF> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Move(ident) => ident.fmt(f),
            Self::Call(call) => call.fmt(f),
            Self::Field(field) => write!(f, "{:?}.{:?}", field.base, field.member),
            Self::Struct(s) => {
                s.type_path.fmt(f)?;
                let mut franz = f.debug_map();
                for (field_name, field_value) in &s.fields {
                    franz.entry(field_name, field_value);
                }
                franz.finish()
            }
            Self::Reference(inner) => write!(f, "&{:?}", inner),
            Self::Lit(lit, neg) => {
                if *neg {
                    write!(f, "-")?;
                }

                lit.fmt(f)
            }
        }
    }
}

/*
impl WExpr<WExprLowCall> {
    pub fn idents(&self) -> Vec<WIdent> {
        match self {
            WExpr::Move(ident) => {
                vec![ident.clone()]
            }
            WExpr::Call(call) => call.idents(),
            WExpr::Field(expr_field) => {
                vec![expr_field.base.clone(), expr_field.member.clone()]
            }
            WExpr::Struct(expr_struct) => expr_struct
                .fields
                .iter()
                .map(|(_name, member)| member.clone())
                .collect(),
            WExpr::Reference(expr_reference) => match expr_reference {
                WExprReference::Ident(ident) => vec![ident.clone()],
                WExprReference::Field(expr_field) => {
                    vec![expr_field.base.clone(), expr_field.member.clone()]
                }
            },
            WExpr::Lit(..) => vec![],
        }
    }
}
*/
