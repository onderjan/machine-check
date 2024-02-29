use syn::{ImplItem, ImplItemFn, ItemImpl, ItemStruct, Path, Stmt};
use syn_path::path;

use crate::{
    util::{
        create_arg, create_expr_call, create_expr_field, create_expr_logical_or, create_expr_path,
        create_field_value, create_ident, create_impl_item_fn, create_item_impl,
        create_path_from_ident, create_path_with_last_generic_type, create_self, create_self_arg,
        create_struct_expr, create_type_path, ArgType,
    },
    MachineError,
};

pub(crate) fn meta_impl(s: &ItemStruct, abstr_type_path: &Path) -> Result<ItemImpl, MachineError> {
    let trait_path = path!(::mck::misc::Meta);
    let trait_path =
        create_path_with_last_generic_type(trait_path, create_type_path(abstr_type_path.clone()));

    let first_fn = proto_first_fn(s, abstr_type_path)?;
    let increment_fn = proto_increment_fn(s, abstr_type_path)?;

    Ok(create_item_impl(
        Some(trait_path),
        create_path_from_ident(s.ident.clone()),
        vec![ImplItem::Fn(first_fn), ImplItem::Fn(increment_fn)],
    ))
}

fn proto_first_fn(s: &ItemStruct, abstr_type_path: &Path) -> Result<ImplItemFn, MachineError> {
    let fn_ident = create_ident("proto_first");

    let self_arg = create_self_arg(ArgType::Reference);
    let return_type = create_type_path(abstr_type_path.clone());

    let mut struct_expr_fields = Vec::new();

    for (index, field) in s.fields.iter().enumerate() {
        let self_field_expr = create_expr_field(create_self(), index, field);
        let init_expr = create_expr_call(
            create_expr_path(path!(::mck::misc::Meta::proto_first)),
            vec![(ArgType::Reference, self_field_expr)],
        );
        struct_expr_fields.push(create_field_value(index, field, init_expr));
    }

    let struct_expr = create_struct_expr(abstr_type_path.clone(), struct_expr_fields);

    Ok(create_impl_item_fn(
        fn_ident,
        vec![self_arg],
        Some(return_type),
        vec![Stmt::Expr(struct_expr, Default::default())],
    ))
}

fn proto_increment_fn(s: &ItemStruct, abstr_type_path: &Path) -> Result<ImplItemFn, MachineError> {
    let fn_ident = create_ident("proto_increment");

    let self_arg = create_self_arg(ArgType::Reference);
    let proto_ident = create_ident("proto");
    let proto_type = create_type_path(abstr_type_path.clone());
    let proto_arg = create_arg(
        ArgType::MutableReference,
        proto_ident.clone(),
        Some(proto_type),
    );

    let return_type = create_type_path(path!(bool));

    let mut result_expr = None;

    for (index, field) in s.fields.iter().enumerate() {
        let fabricated_expr_path = create_expr_path(create_path_from_ident(proto_ident.clone()));

        let self_expr = create_expr_field(create_self(), index, field);
        let fabricated_expr = create_expr_field(fabricated_expr_path, index, field);
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

    Ok(create_impl_item_fn(
        fn_ident,
        vec![self_arg, proto_arg],
        Some(return_type),
        vec![Stmt::Expr(result_expr, None)],
    ))
}
