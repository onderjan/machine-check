use std::collections::HashMap;

use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, Expr, ExprLit, GenericArgument, Ident,
    ImplItemFn, Lit, LitInt, Path, PathArguments, Stmt, Type,
};

use crate::util::{create_ident, create_path_segment, create_type_path};

use super::local::extract_local_ident_with_type;

pub fn find_local_types(impl_item_fn: &ImplItemFn) -> HashMap<Ident, Type> {
    let mut result = HashMap::new();
    // find temporary types
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

pub fn single_bit_type(flavour: &str) -> Type {
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
