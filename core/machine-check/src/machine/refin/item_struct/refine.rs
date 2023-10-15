use syn::{BinOp, Expr, ExprBinary, ImplItem, ImplItemFn, Item, ItemStruct, Path, Stmt};
use syn_path::path;

use crate::machine::{
    refin::rules,
    util::{
        create_arg, create_expr_call, create_expr_field, create_expr_ident, create_expr_path,
        create_ident, create_impl_item_fn, create_item_impl, create_path_from_ident,
        create_path_with_last_generic_type, create_refine_join_stmt, create_self, create_self_arg,
        create_type_path, ArgType,
    },
};

pub fn refine_impl(item_struct: &ItemStruct) -> Result<Item, anyhow::Error> {
    let refin_fn = apply_refin_fn(item_struct)?;
    let join_fn = apply_join_fn(item_struct)?;
    let decay_fn = force_decay_fn(item_struct)?;

    let abstr_type_path =
        rules::abstract_type().convert_path(create_path_from_ident(item_struct.ident.clone()))?;
    let refine_trait: Path = path!(::mck::refin::Refine);
    let refine_trait =
        create_path_with_last_generic_type(refine_trait, create_type_path(abstr_type_path));

    Ok(Item::Impl(create_item_impl(
        Some(refine_trait),
        create_path_from_ident(item_struct.ident.clone()),
        vec![
            ImplItem::Fn(refin_fn),
            ImplItem::Fn(join_fn),
            ImplItem::Fn(decay_fn),
        ],
    )))
}

fn apply_join_fn(s: &ItemStruct) -> anyhow::Result<ImplItemFn> {
    let fn_ident = create_ident("apply_join");

    let self_input = create_self_arg(ArgType::MutableReference);
    let other_ident = create_ident("other");
    let other_input = create_arg(ArgType::Reference, other_ident.clone(), None);

    let mut join_stmts = Vec::new();
    for (index, field) in s.fields.iter().enumerate() {
        let left = create_expr_field(create_self(), index, field);
        let right = create_expr_field(create_expr_ident(other_ident.clone()), index, field);
        let join_stmt = create_refine_join_stmt(left, right);
        join_stmts.push(join_stmt);
    }

    Ok(create_impl_item_fn(
        fn_ident,
        vec![self_input, other_input],
        None,
        join_stmts,
    ))
}

fn force_decay_fn(s: &ItemStruct) -> anyhow::Result<ImplItemFn> {
    let fn_ident = create_ident("force_decay");

    let self_arg = create_self_arg(ArgType::Reference);

    let target_ident = create_ident("target");
    let target_type = create_type_path(
        rules::abstract_type().convert_path(create_path_from_ident(s.ident.clone()))?,
    );
    let target_arg = create_arg(
        ArgType::MutableReference,
        target_ident.clone(),
        Some(target_type),
    );

    let mut stmts = Vec::new();
    for (index, field) in s.fields.iter().enumerate() {
        let decay_arg = create_expr_field(create_self(), index, field);
        let target_arg = create_expr_field(create_expr_ident(target_ident.clone()), index, field);
        let stmt = Stmt::Expr(
            create_expr_call(
                create_expr_path(path!(::mck::refin::Refine::force_decay)),
                vec![
                    (ArgType::Reference, decay_arg),
                    (ArgType::MutableReference, target_arg),
                ],
            ),
            Some(Default::default()),
        );
        stmts.push(stmt);
    }

    Ok(create_impl_item_fn(
        fn_ident,
        vec![self_arg, target_arg],
        None,
        stmts,
    ))
}

fn apply_refin_fn(s: &ItemStruct) -> anyhow::Result<ImplItemFn> {
    let fn_ident = create_ident("apply_refin");

    let self_input = create_self_arg(ArgType::MutableReference);
    let offer_ident = create_ident("offer");
    let offer_input = create_arg(ArgType::Reference, offer_ident.clone(), None);

    let mut result_expr: Option<Expr> = None;
    for (index, field) in s.fields.iter().enumerate() {
        let left = create_expr_field(create_self(), index, field);
        let right = create_expr_field(create_expr_ident(offer_ident.clone()), index, field);

        let expr = create_expr_call(
            create_expr_path(path!(::mck::refin::Refine::apply_refin)),
            vec![
                (ArgType::MutableReference, left),
                (ArgType::Reference, right),
            ],
        );

        if let Some(previous_expr) = result_expr.take() {
            // short-circuiting or for simplicity
            result_expr = Some(Expr::Binary(ExprBinary {
                attrs: vec![],
                left: Box::new(previous_expr),
                op: BinOp::Or(Default::default()),
                right: Box::new(expr),
            }))
        } else {
            result_expr = Some(expr);
        }
    }

    // if there are no fields, return false
    let result_expr = result_expr.unwrap_or(create_expr_path(path!(false)));

    let return_type = create_type_path(path!(bool));

    Ok(create_impl_item_fn(
        fn_ident,
        vec![self_input, offer_input],
        Some(return_type),
        vec![Stmt::Expr(result_expr, None)],
    ))
}
