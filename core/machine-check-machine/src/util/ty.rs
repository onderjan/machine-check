use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, Expr, ExprLit, GenericArgument, Lit,
    LitInt, Path, PathArguments, ReturnType, Type, TypePath, TypeReference, TypeTuple,
};

use super::{create_ident, create_path_segment, ArgType};

pub fn create_type_path(path: Path) -> Type {
    Type::Path(TypePath { qself: None, path })
}
pub fn create_type_reference(mutable: bool, ty: Type) -> Type {
    let mutability = if mutable {
        Some(Default::default())
    } else {
        None
    };
    Type::Reference(TypeReference {
        and_token: Default::default(),
        lifetime: Default::default(),
        mutability,
        elem: Box::new(ty),
    })
}
pub fn create_converted_type(arg_ty: ArgType, ty: Type) -> Type {
    match arg_ty {
        ArgType::Normal => ty,
        ArgType::Reference => create_type_reference(false, ty),
        ArgType::MutableReference => create_type_reference(true, ty),
    }
}

pub fn create_tuple_type(types: Vec<Type>) -> Type {
    Type::Tuple(TypeTuple {
        paren_token: Default::default(),
        elems: Punctuated::from_iter(types),
    })
}

pub fn create_type_from_return_type(return_type: &ReturnType) -> Type {
    match return_type {
        ReturnType::Default => Type::Tuple(TypeTuple {
            paren_token: Default::default(),
            elems: Punctuated::new(),
        }),
        ReturnType::Type(_, ty) => *ty.clone(),
    }
}

pub fn extract_type_path(ty: &Type) -> Option<Path> {
    if let Type::Path(path) = ty {
        Some(path.path.clone())
    } else {
        None
    }
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
