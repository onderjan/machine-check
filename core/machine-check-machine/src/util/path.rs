use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, token::Comma, AngleBracketedGenericArguments, GenericArgument, Ident,
    Path, PathArguments, PathSegment, Token, Type,
};

pub fn create_ident(name: &str) -> Ident {
    // TODO: fix spans
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

pub fn extract_path_ident(path: &Path) -> Option<&Ident> {
    if path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments[0].arguments.is_none()
    {
        Some(&path.segments[0].ident)
    } else {
        None
    }
}

pub fn path_matches_global_names(path: &Path, names: &[&'static str]) -> bool {
    if path.leading_colon.is_none() || path.segments.len() != names.len() {
        return false;
    }
    for (segment, name) in path.segments.iter().zip(names) {
        if segment.ident != name {
            return false;
        }
    }
    true
}

pub fn path_starts_with_global_names(path: &Path, names: &[&'static str]) -> bool {
    if path.leading_colon.is_none() || path.segments.len() < names.len() {
        return false;
    }
    for (segment, name) in path.segments.iter().zip(names) {
        if segment.ident != name {
            return false;
        }
    }
    true
}

pub fn create_angle_bracketed_path_arguments(
    turbofish: bool,
    args: Vec<GenericArgument>,
    span: Span,
) -> PathArguments {
    let colon2_token = if turbofish {
        Some(Token![::](span))
    } else {
        None
    };
    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
        colon2_token,
        lt_token: Token![<](span),
        args: Punctuated::<_, Comma>::from_iter(args),
        gt_token: Token![>](span),
    })
}
