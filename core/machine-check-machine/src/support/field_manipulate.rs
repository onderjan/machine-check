use std::collections::HashSet;

use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, Arm, Expr, ExprLit, ExprMatch, Field,
    GenericArgument, Ident, ImplItem, ImplItemFn, Item, ItemImpl, ItemStruct, Lit, LitInt, LitStr,
    Pat, Path, PathArguments, Stmt, Type,
};
use syn_path::path;

use crate::{
    util::{
        create_arg, create_converted_type, create_expr_call, create_expr_field_named,
        create_expr_ident, create_expr_path, create_ident, create_impl_item_fn, create_item_impl,
        create_pat_wild, create_path_from_ident, create_path_from_name, create_path_segment,
        create_path_with_last_generic_type, create_self, create_self_arg, create_type_path,
        ArgType,
    },
    MachineError,
};

use super::special_trait::{special_trait_impl, SpecialTrait};

pub(crate) fn apply_to_items(items: &mut Vec<Item>, flavour: &str) -> Result<(), MachineError> {
    let mut impls_to_add = Vec::new();

    let mut process_idents = HashSet::<Ident>::new();

    for item in items.iter() {
        let Item::Impl(item_impl) = item else {
            continue;
        };

        if let Type::Path(ty) = item_impl.self_ty.as_ref() {
            if let Some(ident) = ty.path.get_ident() {
                if let Some(SpecialTrait::Input) | Some(SpecialTrait::State) =
                    special_trait_impl(item_impl, flavour)
                {
                    process_idents.insert(ident.clone());
                }
            }
        }
    }

    for item in items.iter() {
        let Item::Struct(item_struct) = item else {
            continue;
        };

        if process_idents.remove(&item_struct.ident) {
            impls_to_add.push(create_field_manipulate_impl(item_struct, flavour));
        }
    }
    items.extend(impls_to_add.into_iter().map(Item::Impl));
    Ok(())
}

pub fn create_field_manipulate_impl(item_struct: &ItemStruct, flavour: &str) -> ItemImpl {
    let mut manipulable_field_idents = Vec::<Ident>::new();

    for field in &item_struct.fields {
        if let Some(manipulable_ident) = field_manipulable_ident(field, flavour) {
            manipulable_field_idents.push(manipulable_ident);
        }
    }

    let get_fn = create_fn(false, &manipulable_field_idents, flavour);
    let get_mut_fn = create_fn(true, &manipulable_field_idents, flavour);

    let trait_path = path!(::mck::misc::FieldManipulate);
    let trait_path = create_path_with_last_generic_type(trait_path, single_bit_type(flavour));

    create_item_impl(
        Some(trait_path),
        create_path_from_ident(item_struct.ident.clone()),
        vec![ImplItem::Fn(get_fn), ImplItem::Fn(get_mut_fn)],
    )
}

fn field_manipulable_ident(field: &Field, flavour: &str) -> Option<Ident> {
    let Some(field_ident) = &field.ident else {
        // do not consider unnamed fields
        return None;
    };

    let Type::Path(path_type) = &field.ty else {
        return None;
    };
    if path_type.qself.is_some() || path_type.path.leading_colon.is_none() {
        return None;
    }
    let mut segments_iter = path_type.path.segments.iter();

    let Some(crate_segment) = segments_iter.next() else {
        return None;
    };
    let Some(flavour_segment) = segments_iter.next() else {
        return None;
    };
    let Some(type_segment) = segments_iter.next() else {
        return None;
    };

    if crate_segment.ident != "mck"
        || flavour_segment.ident != flavour
        || type_segment.ident != "Bitvector"
    {
        return None;
    }

    let PathArguments::AngleBracketed(arguments) = &type_segment.arguments else {
        return None;
    };

    if arguments.args.len() != 1 {
        return None;
    }
    let GenericArgument::Const(Expr::Lit(expr_lit)) = &arguments.args[0] else {
        return None;
    };
    let Lit::Int(lit_int) = &expr_lit.lit else {
        return None;
    };

    let Ok(lit_val) = lit_int.base10_parse::<u32>() else {
        return None;
    };
    if lit_val != 1 {
        return None;
    }

    if segments_iter.next().is_some() {
        return None;
    }

    Some(field_ident.clone())
}

fn create_fn(mutable: bool, manipulable_field_idents: &Vec<Ident>, flavour: &str) -> ImplItemFn {
    let fn_ident: Ident = create_ident(if mutable { "get_mut" } else { "get" });
    let self_arg_ty = if mutable {
        ArgType::MutableReference
    } else {
        ArgType::Reference
    };
    let self_arg = create_self_arg(self_arg_ty.clone());
    let name_ident = create_ident("name");
    let name_arg = create_arg(
        ArgType::Reference,
        name_ident.clone(),
        Some(create_type_path(create_path_from_name("str"))),
    );
    let single_bit_type = create_converted_type(self_arg_ty.clone(), single_bit_type(flavour));
    let option_path = path!(::std::option::Option);
    let option_path = create_path_with_last_generic_type(option_path, single_bit_type);
    let return_type = create_type_path(option_path);

    // add arms
    let mut arms = Vec::new();
    for ident in manipulable_field_idents {
        let name = ident.to_string();
        let self_field = create_expr_field_named(create_self(), ident.clone());
        let some = create_expr_call(
            create_expr_path(path!(::std::option::Option::Some)),
            vec![(self_arg_ty.clone(), self_field)],
        );
        arms.push(Arm {
            attrs: vec![],
            pat: Pat::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Str(LitStr::new(&name, Span::call_site())),
            }),
            guard: Default::default(),
            fat_arrow_token: Default::default(),
            body: Box::new(some),
            comma: Some(Default::default()),
        });
    }

    // add default arm
    arms.push(Arm {
        attrs: vec![],
        pat: create_pat_wild(),
        guard: Default::default(),
        fat_arrow_token: Default::default(),
        body: Box::new(create_expr_path(path!(::std::option::Option::None))),
        comma: Some(Default::default()),
    });

    // create match expr

    let match_expr = Expr::Match(ExprMatch {
        attrs: vec![],
        match_token: Default::default(),
        expr: Box::new(create_expr_ident(name_ident)),
        brace_token: Default::default(),
        arms,
    });

    create_impl_item_fn(
        fn_ident,
        vec![self_arg, name_arg],
        Some(return_type),
        vec![Stmt::Expr(match_expr, None)],
    )
}

fn single_bit_type(flavour: &str) -> Type {
    let mut path = Path {
        leading_colon: Some(Default::default()),
        segments: Punctuated::from_iter(vec![
            create_path_segment(create_ident("mck")),
            create_path_segment(create_ident(flavour)),
            create_path_segment(create_ident("Bitvector")),
        ]),
    };
    path.segments.last_mut().unwrap().arguments =
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: Default::default(),
            lt_token: Default::default(),
            args: Punctuated::from_iter(vec![GenericArgument::Const(Expr::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Int(LitInt::new("1", Span::call_site())),
            }))]),
            gt_token: Default::default(),
        });

    create_type_path(path)
}
