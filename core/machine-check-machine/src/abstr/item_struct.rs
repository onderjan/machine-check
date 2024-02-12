mod phi;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{
    parse_quote, punctuated::Punctuated, spanned::Spanned, Expr, ExprStruct, Generics, ImplItem,
    ImplItemFn, Item, ItemImpl, ItemStruct, Path, PathSegment, Stmt, Token, Type,
};
use syn_path::path;

use crate::{
    abstr::item_struct::phi::phi_impl,
    support::meta_eq::meta_eq_impl,
    util::{
        create_arg, create_assign, create_expr_call, create_expr_field, create_expr_ident,
        create_expr_path, create_field_value, create_ident, create_impl_item_fn,
        create_impl_item_type, create_let_bare, create_path_from_ident, create_path_segment,
        create_path_with_last_generic_type, create_type_path, extract_type_path,
        generate_derive_attribute, path_matches_global_names, path_starts_with_global_names,
        ArgType,
    },
    MachineError,
};

pub fn process_item_struct(mut item_struct: ItemStruct) -> Result<Vec<Item>, MachineError> {
    let mut has_derived_eq = false;
    let mut has_derived_partial_eq = false;
    // remove derives of PartialEq and Eq
    // only if they were derived, we can derive corresponding abstract traits
    for attr in item_struct.attrs.iter_mut() {
        if let syn::Meta::List(meta_list) = &mut attr.meta {
            if meta_list.path.is_ident("derive") {
                let tokens = &meta_list.tokens;
                let punctuated: Punctuated<Path, Token![,]> = parse_quote!(#tokens);
                let mut processed_punctuated: Punctuated<Path, Token![,]> = Punctuated::new();
                for derive in punctuated {
                    // TODO: resolve paths
                    if derive.is_ident("PartialEq") {
                        has_derived_partial_eq = true;
                    } else if derive.is_ident("Eq") {
                        has_derived_eq = true;
                    } else {
                        processed_punctuated.push(derive);
                    }
                }
                meta_list.tokens = processed_punctuated.to_token_stream();
            }
        }
    }

    let abstr_impl = create_abstr(&item_struct)?;

    if has_derived_partial_eq && has_derived_eq {
        // add trait implementations
        let phi_impl = phi_impl(&item_struct)?;

        let meta_eq_impl = meta_eq_impl(&item_struct);

        Ok(vec![
            Item::Struct(item_struct),
            abstr_impl,
            meta_eq_impl,
            phi_impl,
        ])
    } else {
        Ok(vec![Item::Struct(item_struct), abstr_impl])
    }
}

fn create_abstr(item_struct: &ItemStruct) -> Result<Item, MachineError> {
    let span = item_struct.span();

    let mut concr_segments = Punctuated::new();
    concr_segments.push(create_path_segment(Ident::new("super", span)));
    concr_segments.push(create_path_segment(item_struct.ident.clone()));
    let concr_path = Path {
        leading_colon: None,
        segments: concr_segments,
    };
    let concr_ty = create_type_path(concr_path);
    let abstr_path =
        create_path_with_last_generic_type(path!(::mck::abstr::Abstr), concr_ty.clone());

    let from_concrete_fn = ImplItem::Fn(from_concrete_fn(item_struct, concr_ty)?);

    Ok(Item::Impl(ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Token![impl](span),
        generics: Generics::default(),
        trait_: Some((None, abstr_path, Token![for](span))),
        self_ty: Box::new(create_type_path(create_path_from_ident(
            item_struct.ident.clone(),
        ))),
        brace_token: Default::default(),
        items: vec![from_concrete_fn],
    }))
}

fn from_concrete_fn(s: &ItemStruct, concr_ty: Type) -> Result<ImplItemFn, MachineError> {
    let concr_ident = create_ident("concr");
    let concr_arg = create_arg(ArgType::Normal, concr_ident.clone(), Some(concr_ty));

    let mut local_stmts = Vec::new();
    let mut assign_stmts = Vec::new();
    let mut struct_field_values = Vec::new();

    for (index, field) in s.fields.iter().enumerate() {
        let concr_field_expr =
            create_expr_field(create_expr_ident(concr_ident.clone()), index, field);

        let Some(mut concr_field_path) = extract_type_path(&field.ty) else {
            panic!("Expected type path when creating from concrete fn");
        };

        let assign_expr = if path_starts_with_global_names(&concr_field_path, &["mck", "abstr"]) {
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
            Some(field.ty.clone()),
        ));
        assign_stmts.push(create_assign(
            abstr_field_temp_ident.clone(),
            create_expr_call(
                create_expr_path(path!(::mck::abstr::Abstr::from_concrete)),
                vec![(ArgType::Normal, assign_expr)],
            ),
            true,
        ));

        struct_field_values.push(create_field_value(
            index,
            field,
            create_expr_ident(abstr_field_temp_ident),
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
        create_ident("from_concrete"),
        vec![concr_arg],
        Some(create_type_path(path!(Self))),
        local_stmts,
    ))
}
