use std::collections::HashMap;

use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, Expr, ExprLit, GenericArgument, Ident,
    ImplItemFn, Lit, LitInt, Path, PathArguments, Stmt, Type,
};

use crate::util::{
    create_expr_call, create_expr_path, create_ident, create_path_segment, create_type_path,
    extract_pat_ident, path_matches_global_names, ArgType,
};

use super::local::extract_local_ident_with_type;

pub fn find_local_types(impl_item_fn: &ImplItemFn) -> HashMap<Ident, Type> {
    let mut result = HashMap::new();
    // add arguments
    for arg in impl_item_fn.sig.inputs.iter() {
        match arg {
            syn::FnArg::Receiver(receiver) => {
                result.insert(
                    Ident::new("self", receiver.self_token.span),
                    receiver.ty.as_ref().clone(),
                );
            }
            syn::FnArg::Typed(typed) => {
                let ident = extract_pat_ident(&typed.pat);
                result.insert(ident, typed.ty.as_ref().clone());
            }
        }
    }
    // add types in block
    for stmt in impl_item_fn.block.stmts.iter() {
        if let Stmt::Local(local) = stmt {
            let (ident, ty) = extract_local_ident_with_type(local);
            let ty = ty.expect("Expecting all locals to be typed");
            result.insert(ident, ty);
        } else {
            break;
        }
    }
    result
}

fn bitvector_related_type(mut path: Path, num_bits: u32) -> Type {
    path.segments.last_mut().unwrap().arguments =
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: Default::default(),
            lt_token: Default::default(),
            args: Punctuated::from_iter(vec![GenericArgument::Const(Expr::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Int(LitInt::new(&num_bits.to_string(), Span::call_site())),
            }))]),
            gt_token: Default::default(),
        });

    create_type_path(path)
}

#[allow(dead_code)]
pub fn bitvector_type(flavour: &str, num_bits: u32) -> Type {
    let path = Path {
        leading_colon: Some(Default::default()),
        segments: Punctuated::from_iter(vec![
            create_path_segment(create_ident("mck")),
            create_path_segment(create_ident(flavour)),
            create_path_segment(create_ident("Bitvector")),
        ]),
    };
    bitvector_related_type(path, num_bits)
}

pub fn machine_check_bitvector_type(num_bits: u32) -> Type {
    let path = Path {
        leading_colon: Some(Default::default()),
        segments: Punctuated::from_iter(vec![
            create_path_segment(create_ident("machine_check")),
            create_path_segment(create_ident("Bitvector")),
        ]),
    };

    bitvector_related_type(path, num_bits)
}

fn bitvector_related_new(mut path: Path, num_bits: u32, value: &str) -> Expr {
    let bitvector_segment_index = path.segments.len() - 2;
    path.segments[bitvector_segment_index].arguments =
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: Some(Default::default()),
            lt_token: Default::default(),
            args: Punctuated::from_iter(vec![GenericArgument::Const(Expr::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Int(LitInt::new(&num_bits.to_string(), Span::call_site())),
            }))]),
            gt_token: Default::default(),
        });

    let value_expr = Expr::Lit(ExprLit {
        attrs: vec![],
        lit: Lit::Int(LitInt::new(value, Span::call_site())),
    });

    create_expr_call(create_expr_path(path), vec![(ArgType::Normal, value_expr)])
}

#[allow(dead_code)]
pub fn bitvector_new(flavour: &str, num_bits: u32, value: &str) -> Expr {
    let path = Path {
        leading_colon: Some(Default::default()),
        segments: Punctuated::from_iter(vec![
            create_path_segment(create_ident("mck")),
            create_path_segment(create_ident(flavour)),
            create_path_segment(create_ident("Bitvector")),
            create_path_segment(create_ident("new")),
        ]),
    };

    bitvector_related_new(path, num_bits, value)
}

pub fn machine_check_bitvector_new(num_bits: u32, value: &str) -> Expr {
    let path = Path {
        leading_colon: Some(Default::default()),
        segments: Punctuated::from_iter(vec![
            create_path_segment(create_ident("machine_check")),
            create_path_segment(create_ident("Bitvector")),
            create_path_segment(create_ident("new")),
        ]),
    };

    bitvector_related_new(path, num_bits, value)
}

pub fn boolean_type(flavour: &str) -> Type {
    let path = Path {
        leading_colon: Some(Default::default()),
        segments: Punctuated::from_iter(vec![
            create_path_segment(create_ident("mck")),
            create_path_segment(create_ident(flavour)),
            create_path_segment(create_ident("Boolean")),
        ]),
    };
    create_type_path(path)
}

pub fn is_machine_check_bitvector_related_path(path: &Path) -> bool {
    path_matches_global_names(path, &["machine_check", "Bitvector"])
        || path_matches_global_names(path, &["machine_check", "Unsigned"])
        || path_matches_global_names(path, &["machine_check", "Signed"])
}

pub fn is_concr_bitvector_related_path(path: &Path) -> bool {
    path_matches_global_names(path, &["mck", "concr", "Bitvector"])
        || path_matches_global_names(path, &["mck", "concr", "Unsigned"])
        || path_matches_global_names(path, &["mck", "concr", "Signed"])
}
