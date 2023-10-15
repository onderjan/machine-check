use syn::{
    ImplItem, ImplItemFn, ItemImpl, ItemStruct, Path, PathArguments, PathSegment, Stmt, Type,
};
use syn_path::path;

use crate::machine::util::{
    create_expr_call, create_expr_path, create_ident, create_impl_item_fn, create_impl_item_type,
    create_item_impl, create_self_arg, create_type_path, ArgType,
};

pub fn refinable_impl(s: &ItemStruct) -> ItemImpl {
    let refine_type_path = Path::from(s.ident.clone());
    let refine_type = create_type_path(refine_type_path.clone());
    let mut abstr_path = refine_type_path;
    abstr_path.segments.insert(
        0,
        PathSegment {
            ident: create_ident("super"),
            arguments: PathArguments::None,
        },
    );

    let refin_type = create_impl_item_type(create_ident("Refin"), refine_type.clone());

    let trait_path = path!(::mck::refin::Refinable);
    create_item_impl(
        Some(trait_path),
        abstr_path,
        vec![
            ImplItem::Type(refin_type),
            ImplItem::Fn(clean_refin_fn(refine_type)),
        ],
    )
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
