use quote::ToTokens;
use syn::{
    punctuated::Punctuated, Expr, ExprField, ExprIndex, ExprLit, ExprMacro, ExprStruct, FieldValue,
    Ident, ImplItemFn, Lit, LitInt, Macro, MacroDelimiter, Receiver, Stmt, Token,
};
use syn_path::path;

use crate::{
    context::WLowContext,
    util::{
        create_arg, create_assign, create_expr_call, create_expr_ident, create_expr_path,
        create_expr_reference, create_ident, create_impl_item_fn, create_let, create_let_bare,
        create_type_path, create_type_reference, ArgType,
    },
    wir::{IntoTypedSyn, WItemStruct},
};

pub fn from_runtime_fn(item_struct: &WItemStruct, ctx: &WLowContext) -> ImplItemFn {
    let span = item_struct.span().first();
    let runtime_ident = Ident::new("__mck_runtime", span);
    let runtime_ty =
        create_type_reference(false, create_type_path(path!(::mck::abstr::AbstractValue)));
    let runtime_arg = create_arg(ArgType::Normal, runtime_ident.clone(), Some(runtime_ty));

    let mut local_stmts = Vec::new();

    local_stmts.push(create_let(
        runtime_ident.clone(),
        create_expr_call(
            create_expr_path(path!(::mck::abstr::AbstractValue::expect_struct)),
            vec![(ArgType::Normal, create_expr_ident(runtime_ident.clone()))],
        ),
        None,
    ));

    let mut assign_stmts = Vec::new();
    let mut struct_field_values: Vec<FieldValue> = Vec::new();

    for (index, (field_name, field)) in item_struct.fields.iter().enumerate() {
        let index_expr = Expr::Lit(ExprLit {
            attrs: vec![],
            lit: Lit::Int(LitInt::new(index.to_string().as_str(), span)),
        });

        let indexed_expr = Expr::Index(ExprIndex {
            attrs: vec![],
            expr: Box::new(create_expr_ident(runtime_ident.clone())),
            bracket_token: Default::default(),
            index: Box::new(index_expr),
        });

        let assign_expr = create_expr_reference(false, indexed_expr);

        let our_field_temp_ident = create_ident(&format!("__mck_into_abstr_{}", index));
        local_stmts.push(create_let_bare(
            our_field_temp_ident.clone(),
            Some(
                field
                    .ty
                    .clone()
                    .into_typed_syn(&|type_id| ctx.id_syn_type(type_id)),
            ),
        ));
        assign_stmts.push(create_assign(
            our_field_temp_ident.clone(),
            create_expr_call(
                create_expr_path(path!(::mck::abstr::Abstr::from_runtime)),
                vec![(ArgType::Normal, assign_expr)],
            ),
            true,
        ));

        struct_field_values.push(FieldValue {
            attrs: Vec::new(),
            member: syn::Member::Named(field_name.to_syn()),
            colon_token: Some(Token![:](span)),
            expr: create_expr_ident(our_field_temp_ident),
        });
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
        create_ident("from_runtime"),
        vec![runtime_arg],
        Some(create_type_path(path!(Self))),
        local_stmts,
    )
}

pub fn to_runtime_fn(item_struct: &WItemStruct) -> ImplItemFn {
    let span = item_struct.span().first();

    let self_ident = Ident::new("self", span);

    let self_arg = syn::FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: Some((Token![&](span), None)),
        mutability: None,
        self_token: Token![self](span),
        colon_token: None,
        ty: Box::new(create_type_reference(false, create_type_path(path!(Self)))),
    });

    let mut local_stmts = Vec::new();

    let mut exprs: Punctuated<Expr, syn::token::Comma> = Punctuated::new();

    for (field_name, _field) in &item_struct.fields {
        let field_expr = Expr::Field(ExprField {
            attrs: Vec::new(),
            base: Box::new(create_expr_ident(self_ident.clone())),
            dot_token: Token![.](span),
            member: syn::Member::Named(field_name.to_syn()),
        });

        let reference_expr = create_expr_reference(false, field_expr);

        let runtime_expr = create_expr_call(
            create_expr_path(path!(::mck::abstr::Abstr::to_runtime)),
            vec![(ArgType::Normal, reference_expr)],
        );

        exprs.push(runtime_expr);
    }

    let vec_expr = Expr::Macro(ExprMacro {
        attrs: vec![],
        mac: Macro {
            path: path!(::std::vec),
            bang_token: Token![!](span),
            delimiter: MacroDelimiter::Bracket(Default::default()),
            tokens: exprs.into_token_stream(),
        },
    });

    let value_expr = create_expr_call(
        create_expr_path(path!(::mck::abstr::AbstractValue::Struct)),
        vec![(ArgType::Normal, vec_expr)],
    );
    local_stmts.push(Stmt::Expr(value_expr, None));

    create_impl_item_fn(
        create_ident("to_runtime"),
        vec![self_arg],
        Some(create_type_path(path!(::mck::abstr::AbstractValue))),
        local_stmts,
    )
}
