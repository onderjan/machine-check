use syn::{
    punctuated::Punctuated, BinOp, Expr, ExprBinary, ExprCall, ExprField, ExprPath, ExprReference,
    Ident, Member, Path,
};
use syn_path::path;

use super::{create_path_from_ident, ArgType};

pub fn create_expr_field_named(base: Expr, ident: Ident) -> Expr {
    Expr::Field(ExprField {
        attrs: vec![],
        base: Box::new(base),
        dot_token: Default::default(),
        member: Member::Named(ident),
    })
}

pub fn create_expr_logical_and(left: Expr, right: Expr) -> Expr {
    Expr::Binary(ExprBinary {
        attrs: vec![],
        left: Box::new(left),
        op: BinOp::And(Default::default()),
        right: Box::new(right),
    })
}

pub fn create_expr_call(func: Expr, args: Vec<(ArgType, Expr)>) -> Expr {
    let args_iter = args.into_iter().map(|(arg_ty, expr)| match arg_ty {
        ArgType::Normal => expr,
        ArgType::Reference => create_expr_reference(false, expr),
        ArgType::MutableReference => create_expr_reference(true, expr),
    });

    Expr::Call(ExprCall {
        attrs: vec![],
        func: Box::new(func),
        paren_token: Default::default(),
        args: Punctuated::from_iter(args_iter),
    })
}

pub fn create_expr_path(path: Path) -> Expr {
    Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path,
    })
}

pub fn create_expr_ident(ident: Ident) -> Expr {
    create_expr_path(create_path_from_ident(ident))
}

pub fn create_self() -> Expr {
    create_expr_path(path!(self))
}

pub fn create_expr_reference(mutable: bool, expr: Expr) -> Expr {
    let mutability = if mutable {
        Some(Default::default())
    } else {
        None
    };
    Expr::Reference(ExprReference {
        attrs: vec![],
        and_token: Default::default(),
        mutability,
        expr: Box::new(expr),
    })
}
