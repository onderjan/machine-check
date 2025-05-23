use syn::{ImplItem, ImplItemFn, ItemImpl, Stmt};
use syn_path::path;

use crate::{
    util::{
        create_arg, create_expr_call, create_expr_field_named, create_expr_logical_or,
        create_expr_path, create_field_value_ident, create_ident, create_impl_item_fn,
        create_item_impl, create_path_from_ident, create_path_with_last_generic_type, create_self,
        create_self_arg, create_struct_expr, create_type_path, ArgType,
    },
    wir::{WElementaryType, WItemStruct, WPath},
};

pub(crate) fn meta_impl(
    item_struct: &WItemStruct<WElementaryType>,
    abstr_type_path: &WPath,
) -> ItemImpl {
    // Meta implementation which allows iteration over the forward values produced by this backward value
    let trait_path = path!(::mck::misc::Meta);
    let trait_path = create_path_with_last_generic_type(
        trait_path,
        create_type_path(abstr_type_path.clone().into()),
    );

    let first_fn = proto_first_fn(item_struct, abstr_type_path);
    let increment_fn = proto_increment_fn(item_struct, abstr_type_path);

    create_item_impl(
        Some(trait_path),
        item_struct.ident.clone().into_path().into(),
        vec![ImplItem::Fn(first_fn), ImplItem::Fn(increment_fn)],
    )
}

fn proto_first_fn(
    item_struct: &WItemStruct<WElementaryType>,
    abstr_type_path: &WPath,
) -> ImplItemFn {
    // just initialize each field to proto first
    let fn_ident = create_ident("proto_first");

    let self_arg = create_self_arg(ArgType::Reference);
    let return_type = create_type_path(abstr_type_path.clone().into());

    let mut struct_expr_fields = Vec::new();

    for field in &item_struct.fields {
        let self_field_expr = create_expr_field_named(create_self(), field.ident.clone().into());
        let init_expr = create_expr_call(
            create_expr_path(path!(::mck::misc::Meta::proto_first)),
            vec![(ArgType::Reference, self_field_expr)],
        );
        struct_expr_fields.push(create_field_value_ident(
            field.ident.clone().into(),
            init_expr,
        ));
    }

    let struct_expr = create_struct_expr(abstr_type_path.clone().into(), struct_expr_fields);

    create_impl_item_fn(
        fn_ident,
        vec![self_arg],
        Some(return_type),
        vec![Stmt::Expr(struct_expr, Default::default())],
    )
}

fn proto_increment_fn(
    item_struct: &WItemStruct<WElementaryType>,
    abstr_type_path: &WPath,
) -> ImplItemFn {
    // increment the first field which is able to be incremented
    // return whether we were able to increment some field
    let fn_ident = create_ident("proto_increment");

    let self_arg = create_self_arg(ArgType::Reference);
    let proto_ident = create_ident("proto");
    let proto_type = create_type_path(abstr_type_path.clone().into());
    let proto_arg = create_arg(
        ArgType::MutableReference,
        proto_ident.clone(),
        Some(proto_type),
    );

    let return_type = create_type_path(path!(bool));

    let mut result_expr = None;

    for field in &item_struct.fields {
        let fabricated_expr_path = create_expr_path(create_path_from_ident(proto_ident.clone()));

        let self_expr = create_expr_field_named(create_self(), field.ident.clone().into());
        let fabricated_expr =
            create_expr_field_named(fabricated_expr_path, field.ident.clone().into());
        let func_expr = create_expr_path(path!(::mck::misc::Meta::proto_increment));
        let expr = create_expr_call(
            func_expr,
            vec![
                (ArgType::Reference, self_expr),
                (ArgType::MutableReference, fabricated_expr),
            ],
        );
        if let Some(previous_expr) = result_expr.take() {
            // short-circuiting or for simplicity
            result_expr = Some(create_expr_logical_or(previous_expr, expr))
        } else {
            result_expr = Some(expr);
        }
    }

    // if there are no fields, return false
    let result_expr = result_expr.unwrap_or(create_expr_path(path!(false)));

    create_impl_item_fn(
        fn_ident,
        vec![self_arg, proto_arg],
        Some(return_type),
        vec![Stmt::Expr(result_expr, None)],
    )
}
