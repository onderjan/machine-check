use proc_macro2::Ident;
use syn::{Attribute, Local, MetaNameValue, Pat, Stmt, Type};
use syn_path::path;

use crate::util::{create_expr_ident, create_local_bare};

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

pub fn create_let_with_original(tmp_ident: Ident, orig_ident: Ident, ty: Option<Type>) -> Stmt {
    // add attribute that identifies the original ident
    let mut local = create_local_bare(tmp_ident, ty);
    local.attrs.push(Attribute {
        pound_token: Default::default(),
        style: syn::AttrStyle::Outer,
        bracket_token: Default::default(),
        meta: syn::Meta::NameValue(MetaNameValue {
            path: path!(::mck::attr::tmp_original),
            eq_token: Default::default(),
            value: create_expr_ident(orig_ident),
        }),
    });
    Stmt::Local(local)
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
