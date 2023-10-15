use proc_macro2::Span;
use syn::{Ident, Path, PathSegment};

pub fn create_ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

pub fn create_path_from_ident(ident: Ident) -> Path {
    Path::from(ident)
}

pub fn create_path_from_name(name: &str) -> Path {
    create_path_from_ident(create_ident(name))
}

pub fn create_path_segment(ident: Ident) -> PathSegment {
    PathSegment {
        ident,
        arguments: syn::PathArguments::None,
    }
}
