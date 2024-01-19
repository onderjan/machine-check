use proc_macro2::Ident;
use syn::{Attribute, MetaNameValue, Stmt};
use syn_path::path;

use crate::util::{create_expr_ident, create_local};

pub fn create_temporary_ident(prefix: &str, orig_ident: &Ident) -> Ident {
    let orig_ident_string = orig_ident.to_string();
    // TODO: stripping currently cannot be done due to dependence on identifier
    // make sure everything is prefixed by __mck_ only once at the start
    /*let stripped_ident_str = orig_ident_string
        .strip_prefix("__mck_")
        .unwrap_or(&orig_ident_string);*/
    let stripped_ident_str = &orig_ident_string;

    Ident::new(
        &format!("__mck_{}_{}", prefix, stripped_ident_str),
        orig_ident.span(),
    )
}

pub fn create_temporary_let(tmp_ident: Ident, orig_ident: Ident) -> Stmt {
    // add attribute that identifies the original ident
    let mut local = create_local(tmp_ident);
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
