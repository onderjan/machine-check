use syn::{Expr, Stmt};
use syn_path::path;

use crate::util::{create_expr_call, create_expr_path, ArgType};

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
