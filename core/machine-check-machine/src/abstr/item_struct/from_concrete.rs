use syn::{
    punctuated::Punctuated, spanned::Spanned, Expr, ExprField, ExprStruct, FieldValue, Ident,
    ImplItemFn, Stmt, Token, Type,
};
use syn_path::path;

use crate::{
    util::{
        create_arg, create_assign, create_expr_call, create_expr_ident, create_expr_path,
        create_ident, create_impl_item_fn, create_let_bare, create_type_path,
        path_starts_with_global_names, ArgType,
    },
    wir::{IntoSyn, WElementaryType, WItemStruct},
    Error,
};

pub fn from_concrete_fn(
    item_struct: &WItemStruct<WElementaryType>,
    concr_ty: Type,
) -> Result<ImplItemFn, Error> {
    let concr_ident = create_ident("concr");
    let span = concr_ident.span();
    let concr_arg = create_arg(ArgType::Normal, concr_ident.clone(), Some(concr_ty));

    let mut local_stmts = Vec::new();
    let mut assign_stmts = Vec::new();
    let mut struct_field_values = Vec::new();

    for (index, field) in item_struct.fields.iter().enumerate() {
        let concr_field_expr = Expr::Field(ExprField {
            attrs: Vec::new(),
            base: Box::new(create_expr_ident(concr_ident.clone())),
            dot_token: Token![.](span),
            member: syn::Member::Named(field.ident.to_syn_ident()),
        });

        let mut concr_field_path = field.ty.clone().into_syn_path();

        // TODO: nicer concrete type conversion

        let assign_expr = if path_starts_with_global_names(&concr_field_path, &["mck", "forward"]) {
            concr_field_path.segments[1].ident =
                Ident::new("concr", concr_field_path.segments[1].span());

            let mck_field_temp_ident = create_ident(&format!("__mck_into_mck_{}", index));
            local_stmts.push(create_let_bare(
                mck_field_temp_ident.clone(),
                Some(create_type_path(concr_field_path)),
            ));
            assign_stmts.push(create_assign(
                mck_field_temp_ident.clone(),
                create_expr_call(
                    create_expr_path(path!(::mck::concr::IntoMck::into_mck)),
                    vec![(ArgType::Normal, concr_field_expr)],
                ),
                true,
            ));
            create_expr_ident(mck_field_temp_ident)
        } else {
            concr_field_expr
        };

        let abstr_field_temp_ident = create_ident(&format!("__mck_into_abstr_{}", index));
        local_stmts.push(create_let_bare(
            abstr_field_temp_ident.clone(),
            Some(field.ty.clone().into_syn()),
        ));
        assign_stmts.push(create_assign(
            abstr_field_temp_ident.clone(),
            create_expr_call(
                create_expr_path(path!(::mck::abstr::Abstr::from_concrete)),
                vec![(ArgType::Normal, assign_expr)],
            ),
            true,
        ));

        struct_field_values.push(FieldValue {
            attrs: Vec::new(),
            member: syn::Member::Named(field.ident.to_syn_ident()),
            colon_token: Some(Token![:](span)),
            expr: create_expr_ident(abstr_field_temp_ident),
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

    Ok(create_impl_item_fn(
        create_ident("from_concrete"),
        vec![concr_arg],
        Some(create_type_path(path!(Self))),
        local_stmts,
    ))
}
