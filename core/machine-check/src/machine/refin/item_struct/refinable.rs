use syn::{ImplItem, ImplItemFn, ItemImpl, ItemStruct, Stmt, Type};
use syn_path::path;

use crate::machine::{
    refin::rules,
    util::{
        create_expr_call, create_expr_path, create_ident, create_impl_item_fn,
        create_impl_item_type, create_item_impl, create_path_from_ident, create_self_arg,
        create_type_path, ArgType,
    },
};

pub fn refinable_impl(s: &ItemStruct) -> Result<ItemImpl, anyhow::Error> {
    let refine_type_path =
        rules::refinement_type().convert_path(create_path_from_ident(s.ident.clone()))?;
    let refine_type = create_type_path(refine_type_path);
    let abstr_type_path =
        rules::abstract_type().convert_path(create_path_from_ident(s.ident.clone()))?;

    let refin_type = create_impl_item_type(create_ident("Refin"), refine_type.clone());

    let trait_path = path!(::mck::refin::Refinable);
    Ok(create_item_impl(
        Some(trait_path),
        abstr_type_path,
        vec![
            ImplItem::Type(refin_type),
            ImplItem::Fn(clean_refin_fn(refine_type)),
        ],
    ))
}

pub fn clean_refin_fn(refine_type: Type) -> ImplItemFn {
    let self_arg = create_self_arg(ArgType::Reference);

    let expr = create_expr_call(
        create_expr_path(path!(::std::default::Default::default)),
        vec![],
    );

    create_impl_item_fn(
        create_ident("clean_refin"),
        vec![self_arg],
        Some(refine_type),
        vec![Stmt::Expr(expr, None)],
    )
}
