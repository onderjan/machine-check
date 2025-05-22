use syn::{punctuated::Punctuated, Expr, ExprStruct, ImplItem, ImplItemFn, ItemImpl, Stmt};
use syn_path::path;

use crate::{
    util::{
        create_arg, create_assign, create_expr_call, create_expr_field_ident, create_expr_ident,
        create_expr_path, create_field_value_ident, create_ident, create_impl_item_fn,
        create_item_impl, create_let_bare, create_path_from_ident, create_self, create_self_arg,
        create_type_path, ArgType,
    },
    wir::{IntoSyn, WElementaryType, WItemStruct},
};

pub fn phi_impl(item_struct: &WItemStruct<WElementaryType>) -> ItemImpl {
    let phi_fn = phi_fn(item_struct);
    let uninit_fn = uninit_fn(item_struct);

    create_item_impl(
        Some(path!(::mck::forward::Phi)),
        create_path_from_ident(item_struct.ident.to_syn_ident()),
        vec![ImplItem::Fn(phi_fn), ImplItem::Fn(uninit_fn)],
    )
}

fn phi_fn(s: &WItemStruct<WElementaryType>) -> ImplItemFn {
    // phi each field together
    let self_arg = create_self_arg(ArgType::Normal);
    let other_ident = create_ident("other");
    let other_arg = create_arg(ArgType::Normal, other_ident.clone(), None);

    let mut local_stmts = Vec::new();
    let mut assign_stmts = Vec::new();
    let mut struct_field_values = Vec::new();

    for (index, field) in s.fields.iter().enumerate() {
        // assign our field to a temporary as calls can only take ident arguments
        let self_field_expr = create_expr_field_ident(create_self(), field.ident.to_syn_ident());
        let other_field_expr = create_expr_field_ident(
            create_expr_ident(other_ident.clone()),
            field.ident.to_syn_ident(),
        );
        let self_field_temp_ident = create_ident(&format!("__mck_phi_self_{}", index));
        local_stmts.push(create_let_bare(
            self_field_temp_ident.clone(),
            Some(field.ty.clone().into_syn()),
        ));
        assign_stmts.push(create_assign(
            self_field_temp_ident.clone(),
            self_field_expr,
            true,
        ));

        // assign other field to a temporary
        let other_field_temp_ident = create_ident(&format!("__mck_phi_other_{}", index));
        local_stmts.push(create_let_bare(
            other_field_temp_ident.clone(),
            Some(field.ty.clone().into_syn()),
        ));
        assign_stmts.push(create_assign(
            other_field_temp_ident.clone(),
            other_field_expr,
            true,
        ));

        // phi our and other field together
        let phi_result_expr = create_expr_call(
            create_expr_path(path!(::mck::forward::Phi::phi)),
            vec![
                (ArgType::Normal, create_expr_ident(self_field_temp_ident)),
                (ArgType::Normal, create_expr_ident(other_field_temp_ident)),
            ],
        );
        // put the result value into a new temporary, which will be returned by struct initializer
        let phi_result_ident = create_ident(&format!("__mck_phi_result_{}", index));
        local_stmts.push(create_let_bare(
            phi_result_ident.clone(),
            Some(field.ty.clone().into_syn()),
        ));
        assign_stmts.push(create_assign(
            phi_result_ident.clone(),
            phi_result_expr,
            true,
        ));
        struct_field_values.push(create_field_value_ident(
            field.ident.to_syn_ident(),
            create_expr_ident(phi_result_ident),
        ));
    }
    // the result is an initialized struct
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

    create_impl_item_fn(
        create_ident("phi"),
        vec![self_arg, other_arg],
        Some(create_type_path(path!(Self))),
        local_stmts,
    )
}

fn uninit_fn(s: &WItemStruct<WElementaryType>) -> ImplItemFn {
    // each field is uninitialized (using the Phi uninit function)
    let mut local_stmts = Vec::new();
    let mut assign_stmts = Vec::new();
    let mut struct_field_values = Vec::new();

    for (index, field) in s.fields.iter().enumerate() {
        let uninit_expr =
            create_expr_call(create_expr_path(path!(::mck::forward::Phi::uninit)), vec![]);
        let temp_ident = create_ident(&format!("__mck_phi_{}", index));
        local_stmts.push(create_let_bare(
            temp_ident.clone(),
            Some(field.ty.clone().into_syn()),
        ));
        assign_stmts.push(create_assign(temp_ident.clone(), uninit_expr, true));
        struct_field_values.push(create_field_value_ident(
            field.ident.to_syn_ident(),
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

    create_impl_item_fn(
        create_ident("uninit"),
        vec![],
        Some(create_type_path(path!(Self))),
        local_stmts,
    )
}
