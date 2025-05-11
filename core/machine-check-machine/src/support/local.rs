use proc_macro2::Ident;
use syn::{Local, Pat, Type};

pub fn construct_prefixed_ident(prefix: &str, orig_ident: &Ident) -> Ident {
    let orig_ident_string = orig_ident.to_string();
    // make sure everything is prefixed by __mck_ only once at the start
    let stripped_ident_str = orig_ident_string
        .strip_prefix("__mck_")
        .unwrap_or(&orig_ident_string);

    Ident::new(
        &format!("__mck_{}_{}", prefix, stripped_ident_str),
        orig_ident.span(),
    )
}

pub fn extract_local_ident_with_type(local: &Local) -> (Ident, Option<Type>) {
    let mut ty = None;
    let pat = if let Pat::Type(pat_type) = &local.pat {
        ty = Some(pat_type.ty.as_ref().clone());
        pat_type.pat.as_ref()
    } else {
        &local.pat
    };

    let Pat::Ident(ref pat_ident) = pat else {
        panic!("Unexpected non-ident pattern local {:?}", local);
    };
    (pat_ident.ident.clone(), ty)
}
