use syn::{Expr, ExprAssign, Ident, Local, LocalInit, Pat, PatIdent, PatType, Stmt, Type};

use super::{create_expr_path, create_path_from_ident};

fn create_let_mut_choice(
    mutable: bool,
    left_ident: Ident,
    right_expr: Option<Expr>,
    ty: Option<Type>,
) -> Local {
    let mutability = if mutable {
        Some(Default::default())
    } else {
        None
    };
    let mut left_pat = Pat::Ident(PatIdent {
        attrs: vec![],
        by_ref: None,
        mutability,
        ident: left_ident,
        subpat: None,
    });
    if let Some(ty) = ty {
        left_pat = Pat::Type(PatType {
            attrs: vec![],
            pat: Box::new(left_pat),
            colon_token: Default::default(),
            ty: Box::new(ty),
        });
    }
    let init = right_expr.map(|right_expr| LocalInit {
        eq_token: Default::default(),
        expr: Box::new(right_expr),
        diverge: None,
    });

    Local {
        attrs: vec![],
        let_token: Default::default(),
        pat: left_pat,
        init,
        semi_token: Default::default(),
    }
}

pub fn create_let(left_ident: Ident, right_expr: Expr, ty: Option<Type>) -> Stmt {
    Stmt::Local(create_let_mut_choice(
        false,
        left_ident,
        Some(right_expr),
        ty,
    ))
}

pub fn create_let_mut(left_ident: Ident, right_expr: Expr, ty: Option<Type>) -> Stmt {
    Stmt::Local(create_let_mut_choice(
        true,
        left_ident,
        Some(right_expr),
        ty,
    ))
}

pub fn create_let_bare(ident: Ident, ty: Option<Type>) -> Stmt {
    Stmt::Local(create_let_mut_choice(false, ident, None, ty))
}

pub fn create_let_mut_bare(ident: Ident, ty: Option<Type>) -> Stmt {
    Stmt::Local(create_let_mut_choice(true, ident, None, ty))
}

pub fn create_assign(left_ident: Ident, right_expr: Expr, semicolon: bool) -> Stmt {
    Stmt::Expr(
        create_assign_expr(left_ident, right_expr),
        if semicolon {
            Some(Default::default())
        } else {
            None
        },
    )
}

pub fn create_assign_expr(left_ident: Ident, right_expr: Expr) -> Expr {
    let left_expr = create_expr_path(create_path_from_ident(left_ident));
    Expr::Assign(ExprAssign {
        attrs: vec![],
        left: Box::new(left_expr),
        eq_token: Default::default(),
        right: Box::new(right_expr),
    })
}
