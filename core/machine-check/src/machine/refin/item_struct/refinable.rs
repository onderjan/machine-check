use syn::{ImplItem, ImplItemFn, ItemImpl, ItemStruct, Stmt, Type};
use syn_path::path;

use crate::machine::{
    refin::rules,
    util::{
        create_expr_call, create_expr_path, create_ident, create_impl_item_fn,
        create_impl_item_type, create_item_impl, create_path_from_ident, create_self_arg,
        create_type_path, path_rule, ArgType,
    },
};

pub fn refinable_impl(s: &ItemStruct) -> Result<ItemImpl, anyhow::Error> {
    let mut refine_type_path = create_path_from_ident(s.ident.clone());
    path_rule::apply_to_path(&mut refine_type_path, &rules::refinement_type())?;
    let refine_type = create_type_path(refine_type_path.clone());
    let mut abstr_type_path = create_path_from_ident(s.ident.clone());
    path_rule::apply_to_path(&mut abstr_type_path, &rules::abstract_type())?;

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
