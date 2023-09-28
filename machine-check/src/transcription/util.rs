pub mod path_rule;
pub mod scheme;

use proc_macro2::{Span, TokenStream};
use syn::{
    punctuated::Punctuated,
    token::{Bracket, Comma, Paren},
    Attribute, Expr, ExprAssign, ExprCall, ExprField, ExprPath, ExprTuple, Field, Ident, Index,
    Local, LocalInit, Member, MetaList, Pat, PatIdent, PatTuple, PatType, Path, Stmt, Type,
    TypePath,
};
use syn_path::path;

pub fn create_unit_expr() -> Expr {
    Expr::Tuple(ExprTuple {
        attrs: vec![],
        paren_token: Default::default(),
        elems: Punctuated::new(),
    })
}

pub fn create_pat_tuple(expressions: Punctuated<Pat, Comma>) -> Pat {
    Pat::Tuple(PatTuple {
        attrs: vec![],
        paren_token: Default::default(),
        elems: expressions,
    })
}

pub fn create_expr_tuple(expressions: Punctuated<Expr, Comma>) -> Expr {
    Expr::Tuple(ExprTuple {
        attrs: vec![],
        paren_token: Default::default(),
        elems: expressions,
    })
}
pub fn create_assign_stmt(left: Expr, right: Expr) -> Stmt {
    Stmt::Expr(
        Expr::Assign(ExprAssign {
            attrs: vec![],
            left: Box::new(left),
            eq_token: Default::default(),
            right: Box::new(right),
        }),
        Some(Default::default()),
    )
}

pub fn create_ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

pub fn create_path_from_ident(ident: Ident) -> Path {
    Path::from(ident)
}

pub fn create_path_from_name(name: &str) -> Path {
    create_path_from_ident(create_ident(name))
}

pub fn create_pat_ident(ident: Ident) -> PatIdent {
    PatIdent {
        attrs: vec![],
        by_ref: None,
        mutability: None,
        ident,
        subpat: None,
    }
}

pub fn create_expr_field(base: Expr, index: usize, field: &Field) -> ExprField {
    let member = match &field.ident {
        Some(ident) => Member::Named(ident.clone()),
        None => Member::Unnamed(Index {
            index: index as u32,
            span: Span::call_site(),
        }),
    };

    ExprField {
        attrs: vec![],
        base: Box::new(base),
        dot_token: Default::default(),
        member,
    }
}

pub fn create_expr_call(func: Expr, args: Punctuated<Expr, Comma>) -> ExprCall {
    ExprCall {
        attrs: vec![],
        func: Box::new(func),
        paren_token: Default::default(),
        args,
    }
}

pub fn create_expr_path(path: Path) -> ExprPath {
    ExprPath {
        attrs: vec![],
        qself: None,
        path,
    }
}

pub fn create_type_path(path: Path) -> TypePath {
    TypePath { qself: None, path }
}

pub fn create_let_stmt_from_ident_expr(left_ident: Ident, right_expr: Expr) -> Stmt {
    let left_pat = Pat::Ident(PatIdent {
        attrs: vec![],
        by_ref: None,
        mutability: None,
        ident: left_ident,
        subpat: None,
    });
    create_let_stmt_from_pat_expr(left_pat, right_expr)
}

pub fn create_let_stmt_from_pat_expr(left_pat: Pat, right_expr: Expr) -> Stmt {
    Stmt::Local(Local {
        attrs: vec![],
        let_token: Default::default(),
        pat: left_pat,
        init: Some(LocalInit {
            eq_token: Default::default(),
            expr: Box::new(right_expr),
            diverge: None,
        }),
        semi_token: Default::default(),
    })
}

pub fn generate_derive_attribute(tokens: TokenStream) -> Attribute {
    Attribute {
        pound_token: Default::default(),
        style: syn::AttrStyle::Outer,
        bracket_token: Bracket::default(),
        meta: syn::Meta::List(MetaList {
            path: path![derive],
            delimiter: syn::MacroDelimiter::Paren(Paren::default()),
            tokens,
        }),
    }
}

pub fn generate_let_default_stmt(ident: Ident, ty: Type) -> Stmt {
    Stmt::Local(Local {
        attrs: vec![],
        let_token: Default::default(),
        pat: Pat::Type(PatType {
            attrs: vec![],
            pat: Box::new(Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: Some(Default::default()),
                ident,
                subpat: None,
            })),
            colon_token: Default::default(),
            ty: Box::new(ty),
        }),
        init: Some(LocalInit {
            eq_token: Default::default(),
            expr: Box::new(Expr::Call(ExprCall {
                attrs: vec![],
                func: Box::new(Expr::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: path!(::std::default::Default::default),
                })),
                paren_token: Default::default(),
                args: Punctuated::default(),
            })),
            diverge: None,
        }),
        semi_token: Default::default(),
    })
}
