use std::collections::HashMap;

use syn::{punctuated::Punctuated, Ident, ImplItemFn, Path, Stmt, Type};

use crate::util::{create_ident, create_path_segment, create_type_path, extract_pat_ident};

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
