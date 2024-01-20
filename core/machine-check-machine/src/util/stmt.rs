use syn::{Expr, ExprAssign, Ident, Local, LocalInit, Pat, PatIdent, PatType, Stmt, Type};
use syn_path::path;

use super::{create_expr_call, create_expr_path, create_path_from_ident, ArgType};

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

pub fn create_let(left_ident: Ident, right_expr: Expr) -> Stmt {
    Stmt::Local(create_let_mut_choice(
        false,
        left_ident,
        Some(right_expr),
        None,
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

pub fn create_let_bare(ident: Ident) -> Stmt {
    Stmt::Local(create_let_mut_choice(false, ident, None, None))
}

pub fn create_local(ident: Ident, ty: Option<Type>) -> Local {
    create_let_mut_choice(false, ident, None, ty)
}

pub fn create_assign(left_ident: Ident, right_expr: Expr, semicolon: bool) -> Stmt {
    let left_expr = create_expr_path(create_path_from_ident(left_ident));
    Stmt::Expr(
        Expr::Assign(ExprAssign {
            attrs: vec![],
            left: Box::new(left_expr),
            eq_token: Default::default(),
            right: Box::new(right_expr),
        }),
        if semicolon {
            Some(Default::default())
        } else {
            None
        },
    )
}

pub fn create_refine_join_stmt(left: Expr, right: Expr) -> Stmt {
    Stmt::Expr(
        create_expr_call(
            create_expr_path(path!(::mck::refin::Refine::apply_join)),
            vec![
                (ArgType::MutableReference, left),
                (ArgType::Reference, right),
            ],
        ),
        Some(Default::default()),
    )
}
