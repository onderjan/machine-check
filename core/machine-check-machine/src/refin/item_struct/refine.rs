use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, spanned::Spanned, AngleBracketedGenericArguments, BinOp, Expr,
    ExprBinary, ExprLit, ExprStruct, GenericArgument, ImplItem, ImplItemFn, ItemImpl, ItemStruct,
    Lit, LitInt, Path, PathArguments, Stmt,
};
use syn_path::path;

use crate::{
    refin::util::create_refine_join_stmt,
    support::types::boolean_type,
    util::{
        create_arg, create_assign, create_expr_call, create_expr_field, create_expr_ident,
        create_expr_path, create_field_value, create_ident, create_impl_item_fn, create_item_impl,
        create_let_bare, create_let_mut, create_path_from_ident, create_path_from_name,
        create_path_with_last_generic_type, create_self, create_self_arg, create_type_path,
        ArgType,
    },
    BackwardError,
};

pub(crate) fn refine_impl(
    item_struct: &ItemStruct,
    abstr_type_path: &Path,
) -> Result<ItemImpl, BackwardError> {
    let refin_fn = apply_refin_fn(item_struct)?;
    let join_fn = apply_join_fn(item_struct)?;
    let decay_fn = force_decay_fn(item_struct, abstr_type_path)?;
    let to_condition_fn = to_condition_fn(item_struct)?;
    let clean_fn = mark_creation_fn(item_struct, "clean", path!(::mck::refin::Refine::clean))?;
    let dirty_fn = mark_creation_fn(item_struct, "dirty", path!(::mck::refin::Refine::dirty))?;
    let importance_fn = importance_fn(item_struct)?;

    let refine_trait: Path = path!(::mck::refin::Refine);
    let refine_trait =
        create_path_with_last_generic_type(refine_trait, create_type_path(abstr_type_path.clone()));

    Ok(create_item_impl(
        Some(refine_trait),
        create_path_from_ident(item_struct.ident.clone()),
        vec![
            ImplItem::Fn(refin_fn),
            ImplItem::Fn(join_fn),
            ImplItem::Fn(decay_fn),
            ImplItem::Fn(to_condition_fn),
            ImplItem::Fn(clean_fn),
            ImplItem::Fn(dirty_fn),
            ImplItem::Fn(importance_fn),
        ],
    ))
}

fn apply_join_fn(s: &ItemStruct) -> Result<ImplItemFn, BackwardError> {
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

fn force_decay_fn(s: &ItemStruct, abstr_type_path: &Path) -> Result<ImplItemFn, BackwardError> {
    let fn_ident = create_ident("force_decay");

    let self_arg = create_self_arg(ArgType::Reference);

    let target_ident = create_ident("target");
    let target_type = create_type_path(abstr_type_path.clone());
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

fn apply_refin_fn(s: &ItemStruct) -> Result<ImplItemFn, BackwardError> {
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

fn to_condition_fn(s: &ItemStruct) -> Result<ImplItemFn, BackwardError> {
    let fn_ident = create_ident("to_condition");
    let self_input = create_self_arg(ArgType::Reference);

    let return_type = boolean_type("refin");

    // create an unmarked condition first
    let mut stmts = Vec::new();
    let result_ident = create_ident("__mck_result");
    stmts.push(create_let_mut(
        result_ident.clone(),
        create_expr_call(
            create_expr_path(path!(::mck::refin::Boolean::new_unmarked)),
            vec![],
        ),
        Some(return_type.clone()),
    ));

    // join the condition with results of fields
    for (index, field) in s.fields.iter().enumerate() {
        let field_expr = create_expr_field(create_self(), index, field);
        let right = create_expr_call(
            create_expr_path(path!(::mck::refin::Refine::to_condition)),
            vec![(ArgType::Reference, field_expr)],
        );

        stmts.push(create_refine_join_stmt(
            create_expr_ident(result_ident.clone()),
            right,
        ));
    }

    // if there are no fields, return false
    let result_expr = create_expr_ident(result_ident);

    let mut return_path = path!(::mck::refin::Bitvector);

    return_path.segments.last_mut().unwrap().arguments =
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: Default::default(),
            lt_token: Default::default(),
            args: Punctuated::from_iter(vec![GenericArgument::Const(Expr::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Int(LitInt::new("1", Span::call_site())),
            }))]),
            gt_token: Default::default(),
        });

    stmts.push(Stmt::Expr(result_expr, None));

    Ok(create_impl_item_fn(
        fn_ident,
        vec![self_input],
        Some(return_type),
        stmts,
    ))
}

fn mark_creation_fn(
    s: &ItemStruct,
    name: &str,
    name_path: Path,
) -> Result<ImplItemFn, BackwardError> {
    let mut local_stmts = Vec::new();
    let mut assign_stmts = Vec::new();
    let mut struct_field_values = Vec::new();

    for (index, field) in s.fields.iter().enumerate() {
        let uninit_expr = create_expr_call(create_expr_path(name_path.clone()), vec![]);
        let temp_ident = create_ident(&format!("__mck_{}_{}", name, index));
        local_stmts.push(create_let_bare(temp_ident.clone(), None));
        assign_stmts.push(create_assign(temp_ident.clone(), uninit_expr, true));
        struct_field_values.push(create_field_value(
            index,
            field,
            create_expr_ident(temp_ident),
        ));
    }

    let struct_expr = Expr::Struct(ExprStruct {
        attrs: vec![],
        qself: None,
        path: path!(Self),
        brace_token: Default::default(),
        fields: Punctuated::from_iter(struct_field_values),
        dot2_token: None,
        rest: None,
    });
    local_stmts.extend(assign_stmts);
    local_stmts.push(Stmt::Expr(struct_expr, None));

    Ok(create_impl_item_fn(
        create_ident(name),
        vec![],
        Some(create_type_path(path!(Self))),
        local_stmts,
    ))
}

fn importance_fn(s: &ItemStruct) -> Result<ImplItemFn, BackwardError> {
    let span = s.span();
    let fn_ident = create_ident("importance");

    let result_ident = create_ident("__mck_result");
    let self_input = create_self_arg(ArgType::Reference);

    let importance_ty = create_type_path(create_path_from_name("u8"));

    let mut stmts = Vec::new();
    stmts.push(create_let_mut(
        result_ident.clone(),
        Expr::Lit(ExprLit {
            attrs: vec![],
            lit: Lit::Int(LitInt::new("0", span)),
        }),
        Some(importance_ty.clone()),
    ));
    for (index, field) in s.fields.iter().enumerate() {
        let field_expr = create_expr_field(create_self(), index, field);
        let field_importance = create_expr_call(
            create_expr_path(path!(::mck::refin::Refine::importance)),
            vec![(ArgType::Reference, field_expr)],
        );
        let max_importance = create_expr_call(
            create_expr_path(path!(::std::cmp::max)),
            vec![
                (ArgType::Normal, create_expr_ident(result_ident.clone())),
                (ArgType::Normal, field_importance),
            ],
        );
        stmts.push(create_assign(result_ident.clone(), max_importance, true));
    }
    stmts.push(Stmt::Expr(create_expr_ident(result_ident), None));

    Ok(create_impl_item_fn(
        fn_ident,
        vec![self_input],
        Some(importance_ty),
        stmts,
    ))
}
