use syn::{parse_quote, ImplItem, ImplItemFn, ItemImpl, ItemStruct, Path, Stmt};
use syn_path::path;

use crate::machine::util::{
    create_arg, create_expr_binary_or, create_expr_call, create_expr_field, create_expr_path,
    create_field_value, create_ident, create_impl_item_fn, create_item_impl,
    create_path_from_ident, create_self_arg, create_struct_expr, create_type_path, ArgType,
};

pub fn meta_impl(s: &ItemStruct) -> ItemImpl {
    let s_ident = s.ident.clone();
    let struct_path = Path::from(s.ident.clone());
    let impl_path: Path = parse_quote!(::mck::misc::Meta::<super::#s_ident>);

    let first_fn = proto_first_fn(s);
    let increment_fn = proto_increment_fn(s);

    create_item_impl(
        Some(impl_path),
        struct_path,
        vec![ImplItem::Fn(first_fn), ImplItem::Fn(increment_fn)],
    )
}

fn proto_first_fn(s: &ItemStruct) -> ImplItemFn {
    let fn_ident = create_ident("proto_first");

    let self_arg = create_self_arg(ArgType::Reference);
    let s_ident = s.ident.clone();
    let return_type_path: Path = parse_quote!(super::#s_ident);
    let return_type = create_type_path(return_type_path.clone());

    let mut struct_expr_fields = Vec::new();

    for (index, field) in s.fields.iter().enumerate() {
        let self_field_expr = create_expr_field(create_expr_path(path!(self)), index, field);
        let init_expr = create_expr_call(
            create_expr_path(path!(::mck::misc::Meta::proto_first)),
            vec![(ArgType::Reference, self_field_expr)],
        );
        struct_expr_fields.push(create_field_value(index, field, init_expr));
    }

    let struct_expr = create_struct_expr(return_type_path, struct_expr_fields);

    create_impl_item_fn(
        fn_ident,
        vec![self_arg],
        Some(return_type),
        vec![Stmt::Expr(struct_expr, Default::default())],
    )
}

fn proto_increment_fn(s: &ItemStruct) -> ImplItemFn {
    let fn_ident = create_ident("proto_increment");

    let self_arg = create_self_arg(ArgType::Reference);
    let proto_ident = create_ident("proto");
    let s_ident = s.ident.clone();
    let proto_type_path = parse_quote!(super::#s_ident);
    let proto_type = create_type_path(proto_type_path);
    let proto_arg = create_arg(
        ArgType::MutableReference,
        proto_ident.clone(),
        Some(proto_type),
    );

    let return_type = create_type_path(path!(bool));

    let mut result_expr = None;

    for (index, field) in s.fields.iter().enumerate() {
        let self_expr_path = create_expr_path(path!(self));
        let fabricated_expr_path = create_expr_path(create_path_from_ident(proto_ident.clone()));

        let self_expr = create_expr_field(self_expr_path, index, field);
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
            result_expr = Some(create_expr_binary_or(previous_expr, expr))
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
