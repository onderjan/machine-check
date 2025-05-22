use proc_macro2::Ident;
use syn::{punctuated::Punctuated, Generics, ImplItem, ItemImpl, Path, Token};
use syn_path::path;

use crate::{
    abstr::item_struct::phi::phi_impl,
    support::meta_eq::meta_eq_impl,
    util::{
        create_path_from_ident, create_path_segment, create_path_with_last_generic_type,
        create_type_path,
    },
    wir::{IntoSyn, WElementaryType, WItemStruct},
    Error,
};

use self::from_concrete::from_concrete_fn;

mod from_concrete;
mod phi;

pub fn process_item_struct(
    mut item_struct: WItemStruct<WElementaryType>,
) -> Result<(WItemStruct<WElementaryType>, Vec<ItemImpl>), Error> {
    let mut has_derived_eq = false;
    let mut has_derived_partial_eq = false;
    // look for derives of PartialEq and Eq
    // only if they were derived, we can derive corresponding abstract traits
    let mut passthrough_derives = Vec::new();
    for derive in item_struct.derives.drain(..) {
        let passthrough_names_list = [
            ["std", "clone", "Clone"],
            ["std", "hash", "Hash"],
            ["std", "fmt", "Debug"],
        ];

        if derive.starts_with_absolute(&["std", "cmp", "PartialEq"]) {
            has_derived_partial_eq = true;
        } else if derive.starts_with_absolute(&["std", "cmp", "Eq"]) {
            has_derived_eq = true;
        } else {
            let mut passthrough = false;
            for passthrough_names in &passthrough_names_list {
                if derive.starts_with_absolute(passthrough_names) {
                    passthrough = true;
                    break;
                }
            }
            if passthrough {
                passthrough_derives.push(derive);
            }
        }
    }
    item_struct.derives = passthrough_derives;

    let abstr_impl = create_abstr(&item_struct)?;

    if has_derived_partial_eq && has_derived_eq {
        // add phi and meta-eq implementations
        let phi_impl = phi_impl(&item_struct)?;
        // TODO: rewrite meta-eq impl to use WIR
        let meta_eq_item_struct = item_struct.clone().into_syn();
        let meta_eq_impl = meta_eq_impl(&meta_eq_item_struct);

        Ok((item_struct, vec![abstr_impl, meta_eq_impl, phi_impl]))
    } else {
        Ok((item_struct, vec![abstr_impl]))
    }
}

fn create_abstr(item_struct: &WItemStruct<WElementaryType>) -> Result<ItemImpl, Error> {
    let span = item_struct.ident.span();

    let mut concr_segments = Punctuated::new();
    concr_segments.push(create_path_segment(Ident::new("super", span)));
    concr_segments.push(create_path_segment(item_struct.ident.to_syn_ident()));
    let concr_path = Path {
        leading_colon: None,
        segments: concr_segments,
    };
    let concr_ty = create_type_path(concr_path);
    let abstr_path =
        create_path_with_last_generic_type(path!(::mck::abstr::Abstr), concr_ty.clone());

    let from_concrete_fn = ImplItem::Fn(from_concrete_fn(item_struct, concr_ty)?);

    Ok(ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Token![impl](span),
        generics: Generics::default(),
        trait_: Some((None, abstr_path, Token![for](span))),
        self_ty: Box::new(create_type_path(create_path_from_ident(
            item_struct.ident.to_syn_ident(),
        ))),
        brace_token: Default::default(),
        items: vec![from_concrete_fn],
    })
}
