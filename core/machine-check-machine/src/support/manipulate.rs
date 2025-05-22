use std::collections::HashSet;

use proc_macro2::{Literal, Span};
use quote::ToTokens;
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, Arm, Expr, ExprLit, ExprMacro,
    ExprMatch, Field, Ident, ImplItem, ImplItemFn, Item, ItemImpl, ItemStruct, Lifetime, Lit,
    LitStr, Macro, Pat, Path, PathArguments, PathSegment, Stmt, Token, TraitBound, Type,
    TypeParamBound, TypeReference, TypeTraitObject,
};
use syn_path::path;

use crate::util::{
    create_arg, create_expr_call, create_expr_field_named, create_expr_ident, create_expr_path,
    create_ident, create_impl_item_fn, create_item_impl, create_pat_wild, create_path_from_ident,
    create_path_from_name, create_path_with_last_generic_type, create_self, create_self_arg,
    create_type_path, create_type_reference, path_matches_global_names, ArgType,
};

use super::special_trait::{special_trait_impl, SpecialTrait};

#[derive(Clone, Copy)]
pub enum ManipulateKind {
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

pub(crate) fn apply_to_items(items: &mut Vec<Item>, kind: ManipulateKind) {
    let mut impls_to_add = Vec::new();

    let mut process_idents = HashSet::<Ident>::new();

    for item in items.iter() {
        let Item::Impl(item_impl) = item else {
            continue;
        };

        if let Type::Path(ty) = item_impl.self_ty.as_ref() {
            if let Some(ident) = ty.path.get_ident() {
                if let Some(SpecialTrait::Input) | Some(SpecialTrait::State) =
                    special_trait_impl(item_impl, kind.str())
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
            impls_to_add.push(create_manipulatable_impl(item_struct, kind));
        }
    }
    items.extend(impls_to_add.into_iter().map(Item::Impl));
}

pub fn create_manipulatable_impl(item_struct: &ItemStruct, kind: ManipulateKind) -> ItemImpl {
    let mut manipulable_field_idents = Vec::<Ident>::new();

    for field in &item_struct.fields {
        if let Some(manipulable_ident) = field_manipulatable_ident(field, kind) {
            manipulable_field_idents.push(manipulable_ident);
        }
    }

    let span = item_struct.span();
    let get_fn = create_fn(false, &manipulable_field_idents, kind, span);
    let get_mut_fn = create_fn(true, &manipulable_field_idents, kind, span);
    let create_field_names_fn = create_field_names_fn(&manipulable_field_idents, span);

    let trait_path = kind_path(kind, "Manipulatable", span);

    create_item_impl(
        Some(trait_path),
        create_path_from_ident(item_struct.ident.clone()),
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

fn field_manipulatable_ident(field: &Field, kind: ManipulateKind) -> Option<Ident> {
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
    if path_matches_global_names(&path_type.path, &["mck", kind.str(), "Bitvector"])
        || path_matches_global_names(&path_type.path, &["mck", kind.str(), "Array"])
    {
        Some(field_ident.clone())
    } else {
        None
    }
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
        Some(create_type_path(create_path_from_name("str"))),
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
