use syn::{
    parse_quote, BinOp, Expr, ExprBinary, ImplItem, ImplItemFn, Item, ItemStruct, Path, Stmt, Type,
};
use syn_path::path;

use crate::machine::util::{
    create_arg, create_expr_call, create_expr_field, create_expr_ident, create_expr_path,
    create_ident, create_impl_item_fn, create_item_impl, create_path_from_ident,
    create_refine_join_stmt, create_self, create_self_arg, create_type_path, path_rule, ArgType,
};

use self::{meta::generate_fabricator_impl, refinable::generate_markable_impl};

use super::mark_path_rules;

mod meta;
mod refinable;

pub fn apply_to_struct(
    items: &mut Vec<Item>,
    abstr_struct: &ItemStruct,
) -> Result<(), anyhow::Error> {
    {
        // apply path rules and push struct
        let mut refin_struct = abstr_struct.clone();
        path_rule::apply_to_item_struct(&mut refin_struct, mark_path_rules())?;
        let ident_string = refin_struct.ident.to_string();

        // TODO: add the implementations only for state and input according to traits
        if ident_string.as_str() != "Machine" {
            let fabricator_impl = generate_fabricator_impl(&refin_struct)?;
            let markable_impl = generate_markable_impl(&refin_struct)?;
            // add struct
            items.push(Item::Struct(refin_struct));
            // add implementations
            items.push(Item::Impl(fabricator_impl));
            items.push(Item::Impl(markable_impl));
        } else {
            // add struct
            items.push(Item::Struct(refin_struct));
        }

        if abstr_struct.ident == "Input" || abstr_struct.ident == "State" {
            let s_ident = &abstr_struct.ident;
            let refin_fn = generate_mark_single_fn(abstr_struct)?;
            let join_fn = generate_join_fn(abstr_struct)?;
            let decay_fn = generate_force_decay_fn(abstr_struct)?;
            let refine_trait: Path = parse_quote!(::mck::refin::Refine<super::#s_ident>);
            items.push(Item::Impl(create_item_impl(
                Some(refine_trait),
                create_path_from_ident(abstr_struct.ident.clone()),
                vec![
                    ImplItem::Fn(refin_fn),
                    ImplItem::Fn(join_fn),
                    ImplItem::Fn(decay_fn),
                ],
            )));
        }
        Ok(())
    }
}

fn generate_join_fn(s: &ItemStruct) -> anyhow::Result<ImplItemFn> {
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

fn generate_force_decay_fn(state_struct: &ItemStruct) -> anyhow::Result<ImplItemFn> {
    let fn_ident = create_ident("force_decay");

    let self_arg = create_self_arg(ArgType::Reference);

    let target_ident = create_ident("target");
    let s_ident = &state_struct.ident;
    let target_type = Type::Path(create_type_path(parse_quote!(super::#s_ident)));
    let target_arg = create_arg(
        ArgType::MutableReference,
        target_ident.clone(),
        Some(target_type),
    );

    let mut stmts = Vec::new();
    for (index, field) in state_struct.fields.iter().enumerate() {
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

fn generate_mark_single_fn(s: &ItemStruct) -> anyhow::Result<ImplItemFn> {
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

    let return_type = Type::Path(create_type_path(path!(bool)));

    Ok(create_impl_item_fn(
        fn_ident,
        vec![self_input, offer_input],
        Some(return_type),
        vec![Stmt::Expr(result_expr, None)],
    ))
}
