use proc_macro2::Span;
use syn::{
    punctuated::Punctuated,
    token::{Brace, Paren},
    Expr, ExprCall, ExprField, ExprLit, ExprReference, ExprStruct, FieldValue, Lit, Token, Type,
};

use crate::util::{create_expr_ident, create_expr_path};

use super::{IntoSyn, WIdent, WPath};

#[derive(Clone, Debug, Hash)]
pub enum WExpr<FT: IntoSyn<Type>> {
    Move(WIdent),
    Call(WExprCall<FT>),
    Field(WExprField),
    Struct(WExprStruct<FT>),
    Reference(WExprReference),
    Lit(Lit),
}

#[derive(Clone, Debug, Hash)]
pub struct WExprCall<FT: IntoSyn<Type>> {
    pub fn_path: WPath<FT>,
    pub args: Vec<WCallArg>,
}

impl<FT: IntoSyn<Type>> WExprCall<FT> {
    pub fn span(&self) -> Span {
        // TODO: correct span
        self.fn_path.span()
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
    pub inner: WIdent,
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

impl<FT: IntoSyn<Type>> IntoSyn<Expr> for WExpr<FT> {
    fn into_syn(self) -> Expr {
        let span = Span::call_site();
        match self {
            WExpr::Move(ident) => create_expr_ident(ident.into()),
            WExpr::Call(expr) => {
                let args = Punctuated::from_iter(expr.args.into_iter().map(|arg| match arg {
                    WCallArg::Ident(ident) => create_expr_ident(ident.into()),
                    WCallArg::Literal(lit) => Expr::Lit(ExprLit {
                        attrs: Vec::new(),
                        lit,
                    }),
                }));
                Expr::Call(ExprCall {
                    attrs: Vec::new(),
                    func: Box::new(create_expr_path(expr.fn_path.into())),
                    paren_token: Paren::default(),
                    // TODO args
                    args,
                })
            }
            WExpr::Field(expr) => Expr::Field(ExprField {
                attrs: Vec::new(),
                base: Box::new(create_expr_ident(expr.base.into())),
                dot_token: Token![.](span),
                member: syn::Member::Named(expr.inner.into()),
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
                        member: syn::Member::Named(expr.inner.into()),
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
