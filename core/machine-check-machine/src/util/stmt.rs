use syn::{Expr, ExprAssign, Ident, Local, LocalInit, Pat, PatIdent, Stmt};
use syn_path::path;

use super::{create_expr_call, create_expr_path, create_path_from_ident, ArgType};

fn create_let_mut_choice(mutable: bool, left_ident: Ident, right_expr: Option<Expr>) -> Stmt {
    let mutability = if mutable {
        Some(Default::default())
    } else {
        None
    };
    let left_pat = Pat::Ident(PatIdent {
        attrs: vec![],
        by_ref: None,
        mutability,
        ident: left_ident,
        subpat: None,
    });
    let init = right_expr.map(|right_expr| LocalInit {
        eq_token: Default::default(),
        expr: Box::new(right_expr),
        diverge: None,
    });

    Stmt::Local(Local {
        attrs: vec![],
        let_token: Default::default(),
        pat: left_pat,
        init,
        semi_token: Default::default(),
    })
}

pub fn create_let(left_ident: Ident, right_expr: Expr) -> Stmt {
    create_let_mut_choice(false, left_ident, Some(right_expr))
}

pub fn create_let_mut(left_ident: Ident, right_expr: Expr) -> Stmt {
    create_let_mut_choice(true, left_ident, Some(right_expr))
}

pub fn create_let_bare(ident: Ident) -> Stmt {
    create_let_mut_choice(false, ident, None)
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
