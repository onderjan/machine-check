use std::collections::HashSet;

use proc_macro2::{Literal, Span};
use quote::ToTokens;
use syn::{
    punctuated::Punctuated, token::Comma, Arm, Expr, ExprLit, ExprMacro, ExprMatch, Ident,
    ImplItem, ImplItemFn, ItemImpl, Lifetime, Lit, LitStr, Macro, Pat, Path, PathArguments,
    PathSegment, Stmt, Token, TraitBound, Type, TypeParamBound, TypeReference, TypeTraitObject,
};
use syn_path::path;

use crate::{
    abstr::{WAbstrItemImplTrait, YAbstr},
    refin::{WRefinItemImplTrait, YRefin},
    util::{
        create_arg, create_expr_call, create_expr_field_named, create_expr_ident, create_expr_path,
        create_ident, create_impl_item_fn, create_item_impl, create_pat_wild,
        create_path_from_ident, create_path_with_last_generic_type, create_self, create_self_arg,
        create_type_path, create_type_reference, ArgType,
    },
    wir::{WDescription, WElementaryType, WItemImplTrait},
};

pub(crate) fn for_abstract_description(description: &WDescription<YAbstr>) -> Vec<ItemImpl> {
    let mut impls_to_add = Vec::new();

    let mut process_idents = HashSet::new();

    for item_impl in description.impls.iter() {
        if let Some(WAbstrItemImplTrait { trait_, .. }) = &item_impl.trait_ {
            if matches!(trait_, WItemImplTrait::Machine(_)) {
                for impl_item_type in &item_impl.impl_item_types {
                    if impl_item_type.left_ident.name() == "Input"
                        || impl_item_type.left_ident.name() == "State"
                    {
                        if let Some(right_ident) = impl_item_type.right_path.get_ident() {
                            process_idents.insert(right_ident.clone());
                        }
                    }
                }
            }
        }
    }

    for item_struct in description.structs.iter() {
        if process_idents.remove(&item_struct.ident) {
            let manipulable_field_idents = item_struct
                .fields
                .iter()
                .filter_map(|field| {
                    if is_manipulable(&field.ty) {
                        Some(field.ident.to_syn_ident())
                    } else {
                        None
                    }
                })
                .collect();

            impls_to_add.push(create_manipulatable_impl(
                item_struct.ident.span(),
                item_struct.ident.to_syn_ident(),
                &manipulable_field_idents,
                ManipulateKind::Forward,
            ));
        }
    }
    impls_to_add
}

pub(crate) fn for_refinement_description(description: &WDescription<YRefin>) -> Vec<ItemImpl> {
    let mut impls_to_add = Vec::new();

    let mut process_idents = HashSet::new();

    for item_impl in description.impls.iter() {
        if let Some(WRefinItemImplTrait { trait_, .. }) = &item_impl.trait_ {
            if matches!(trait_, WItemImplTrait::Machine(_)) {
                for impl_item_type in &item_impl.impl_item_types {
                    if impl_item_type.left_ident.name() == "Input"
                        || impl_item_type.left_ident.name() == "State"
                    {
                        if let Some(right_ident) = impl_item_type.right_path.get_ident() {
                            process_idents.insert(right_ident.clone());
                        }
                    }
                }
            }
        }
    }

    for item_struct in description.structs.iter() {
        if process_idents.remove(&item_struct.ident) {
            let manipulable_field_idents = item_struct
                .fields
                .iter()
                .filter_map(|field| {
                    if is_manipulable(field.ty.forward_type()) {
                        Some(field.ident.to_syn_ident())
                    } else {
                        None
                    }
                })
                .collect();

            impls_to_add.push(create_manipulatable_impl(
                item_struct.ident.span(),
                item_struct.ident.to_syn_ident(),
                &manipulable_field_idents,
                ManipulateKind::Backward,
            ));
        }
    }
    impls_to_add
}

#[derive(Clone, Copy)]
enum ManipulateKind {
    Forward,
    Backward,
}

impl ManipulateKind {
    fn str(&self) -> &'static str {
        match self {
            ManipulateKind::Forward => "forward",
            ManipulateKind::Backward => "backward",
        }
    }
}

fn create_manipulatable_impl(
    span: Span,
    ident: Ident,
    manipulable_field_idents: &Vec<Ident>,
    kind: ManipulateKind,
) -> ItemImpl {
    let get_fn = create_fn(false, manipulable_field_idents, kind, span);
    let get_mut_fn = create_fn(true, manipulable_field_idents, kind, span);
    let create_field_names_fn = create_field_names_fn(manipulable_field_idents, span);

    let trait_path = kind_path(kind, "Manipulatable", span);

    create_item_impl(
        Some(trait_path),
        create_path_from_ident(ident),
        vec![
            ImplItem::Fn(get_fn),
            ImplItem::Fn(get_mut_fn),
            ImplItem::Fn(create_field_names_fn),
        ],
    )
}

fn kind_path(kind: ManipulateKind, last_str: &str, span: Span) -> Path {
    Path {
        leading_colon: Some(Token![::](span)),
        segments: Punctuated::from_iter([
            PathSegment {
                ident: Ident::new("mck", span),
                arguments: PathArguments::None,
            },
            PathSegment {
                ident: Ident::new(kind.str(), span),
                arguments: PathArguments::None,
            },
            PathSegment {
                ident: Ident::new(last_str, span),
                arguments: PathArguments::None,
            },
        ]),
    }
}

fn is_manipulable(ty: &WElementaryType) -> bool {
    matches!(
        ty,
        WElementaryType::Bitvector(_) | WElementaryType::Array(_)
    )
}

fn create_fn(
    mutable: bool,
    manipulable_field_idents: &Vec<Ident>,
    kind: ManipulateKind,
    span: Span,
) -> ImplItemFn {
    let fn_ident: Ident = Ident::new(if mutable { "get_mut" } else { "get" }, span);
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
        Some(create_type_path(create_path_from_ident(create_ident(
            "str",
        )))),
    );
    let manip_field_type = create_type_reference(
        mutable,
        Type::TraitObject(TypeTraitObject {
            dyn_token: Some(Token![dyn](span)),
            bounds: Punctuated::from_iter([TypeParamBound::Trait(TraitBound {
                paren_token: None,
                modifier: syn::TraitBoundModifier::None,
                lifetimes: None,
                path: kind_path(kind, "ManipField", span),
            })]),
        }),
    );

    let option_path = path!(::std::option::Option);
    let option_path = create_path_with_last_generic_type(option_path, manip_field_type);
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

fn create_field_names_fn(field_idents: &[Ident], span: Span) -> ImplItemFn {
    let fn_ident: Ident = Ident::new("field_names", span);

    let str_type = create_type_path(path!(::std::primitive::str));
    let str_ref_type = Type::Reference(TypeReference {
        and_token: Token![&](span),
        lifetime: Some(Lifetime::new("'static", span)),
        mutability: None,
        elem: Box::new(str_type),
    });

    let vec_path = path!(::std::vec::Vec);
    let vec_path = create_path_with_last_generic_type(vec_path, str_ref_type);
    let return_type = create_type_path(vec_path);

    let mut punctuated = Punctuated::<Literal, Comma>::new();
    for field_ident in field_idents {
        punctuated.push(Literal::string(&field_ident.to_string()));
    }

    let result_expr = Expr::Macro(ExprMacro {
        attrs: vec![],
        mac: Macro {
            path: path!(::std::vec),
            bang_token: Token![!](span),
            delimiter: syn::MacroDelimiter::Bracket(Default::default()),
            tokens: punctuated.into_token_stream(),
        },
    });

    create_impl_item_fn(
        fn_ident,
        vec![],
        Some(return_type),
        vec![Stmt::Expr(result_expr, None)],
    )
}
