use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, GenericArgument, Ident, Path,
    PathSegment, Type,
};

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

pub fn create_path_with_last_generic_type(path: Path, ty: Type) -> Path {
    // add generic with the abstract type
    let mut path = path;
    path.segments
        .last_mut()
        .expect("Path should have last segment for adding generic")
        .arguments = syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments {
        colon2_token: Default::default(),
        lt_token: Default::default(),
        args: Punctuated::from_iter(vec![GenericArgument::Type(ty)]),
        gt_token: Default::default(),
    });
    path
}

pub fn extract_path_ident(path: Path) -> Ident {
    if path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments[0].arguments.is_none()
    {
        path.segments.into_iter().next().unwrap().ident
    } else {
        panic!("Unexpected non-ident path {:?}", path);
    }
}
