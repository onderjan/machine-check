use syn::{spanned::Spanned, Ident, ImplItem, ItemImpl, ItemStruct, Stmt};
use syn_path::path;

use crate::util::{
    create_arg, create_expr_call, create_expr_field, create_expr_ident, create_expr_logical_and,
    create_expr_path, create_impl_item_fn, create_item_impl, create_path_from_ident, create_self,
    create_self_arg, create_type_path, ArgType,
};

pub fn meta_eq_impl(item_struct: &ItemStruct) -> ItemImpl {
    // two underscores to avoid unused variable warning if there are no fields
    let other_ident = Ident::new("__other", item_struct.span());

    let mut result_expr = None;

    for (index, field) in item_struct.fields.iter().enumerate() {
        let left = create_expr_field(create_self(), index, field);
        let right = create_expr_field(create_expr_ident(other_ident.clone()), index, field);
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
        Ident::new("meta_eq", item_struct.span()),
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
        create_path_from_ident(item_struct.ident.clone()),
        vec![ImplItem::Fn(eq_fn)],
    )
}
