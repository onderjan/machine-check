use syn::{Ident, ImplItem, ItemImpl, Stmt};
use syn_path::path;

use crate::{
    util::{
        create_arg, create_expr_call, create_expr_field_named, create_expr_ident,
        create_expr_logical_and, create_expr_path, create_impl_item_fn, create_item_impl,
        create_path_from_ident, create_self, create_self_arg, create_type_path, ArgType,
    },
    wir::{WElementaryType, WItemStruct},
};

pub fn meta_eq_impl(item_struct: &WItemStruct<WElementaryType>) -> ItemImpl {
    let span = item_struct.ident.span();

    // two underscores to avoid unused variable warning if there are no fields
    let other_ident = Ident::new("__other", span);

    let mut result_expr = None;

    for field in &item_struct.fields {
        let left = create_expr_field_named(create_self(), field.ident.clone().into());
        let right = create_expr_field_named(
            create_expr_ident(other_ident.clone()),
            field.ident.clone().into(),
        );
        let eq_expr = create_expr_call(
            create_expr_path(path!(::mck::misc::MetaEq::meta_eq)),
            vec![(ArgType::Reference, left), (ArgType::Reference, right)],
        );

        if let Some(previous_expr) = result_expr.take() {
            // short-circuiting and for simplicity
            result_expr = Some(create_expr_logical_and(previous_expr, eq_expr));
        } else {
            result_expr = Some(eq_expr);
        }
    }

    let eq_fn = create_impl_item_fn(
        Ident::new("meta_eq", span),
        vec![
            create_self_arg(ArgType::Reference),
            create_arg(ArgType::Reference, other_ident, None),
        ],
        Some(create_type_path(path!(bool))),
        vec![Stmt::Expr(
            result_expr.unwrap_or_else(|| create_expr_path(path!(true))),
            None,
        )],
    );

    create_item_impl(
        Some(path!(::mck::misc::MetaEq)),
        create_path_from_ident(item_struct.ident.clone().into()),
        vec![ImplItem::Fn(eq_fn)],
    )
}
