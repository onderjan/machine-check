use proc_macro2::Ident;
use quote::ToTokens;
use syn::{
    parse::Parser, punctuated::Punctuated, spanned::Spanned, Generics, ImplItem, Item, ItemImpl,
    ItemStruct, Path, Token,
};
use syn_path::path;

use crate::{
    abstr::item_struct::phi::phi_impl,
    support::meta_eq::meta_eq_impl,
    util::{
        create_path_from_ident, create_path_segment, create_path_with_last_generic_type,
        create_type_path, path_matches_global_names,
    },
    Error, ErrorType,
};

use self::from_concrete::from_concrete_fn;

mod from_concrete;
mod phi;

pub fn process_item_struct(mut item_struct: ItemStruct) -> Result<Vec<Item>, Error> {
    let mut has_derived_eq = false;
    let mut has_derived_partial_eq = false;
    // look for derives of PartialEq and Eq
    // only if they were derived, we can derive corresponding abstract traits
    for attr in item_struct.attrs.iter_mut() {
        let syn::Meta::List(meta_list) = &mut attr.meta else {
            continue;
        };
        if !meta_list.path.is_ident("derive") {
            continue;
        }

        let parser = Punctuated::<Path, Token![,]>::parse_terminated;

        let Ok(punctuated) = parser.parse2(meta_list.tokens.clone()) else {
            // could not be parsed, skip attribude
            continue;
        };
        let mut processed_punctuated: Punctuated<Path, Token![,]> = Punctuated::new();
        for derive in punctuated {
            let passthrough_names_list = [
                ["std", "clone", "Clone"],
                ["std", "hash", "Hash"],
                ["std", "fmt", "Debug"],
            ];

            if path_matches_global_names(&derive, &["std", "cmp", "PartialEq"]) {
                has_derived_partial_eq = true;
            } else if path_matches_global_names(&derive, &["std", "cmp", "Eq"]) {
                has_derived_eq = true;
            } else {
                let mut passthrough = false;
                for passthrough_names in &passthrough_names_list {
                    if path_matches_global_names(&derive, passthrough_names) {
                        passthrough = true;
                        break;
                    }
                }
                if passthrough {
                    processed_punctuated.push(derive);
                } else {
                    return Err(Error::new(
                        ErrorType::ForwardConversionError(String::from(
                            "Unable to passthrough derive attribute",
                        )),
                        derive.span(),
                    ));
                }
            }
        }
        meta_list.tokens = processed_punctuated.to_token_stream();
    }

    let abstr_impl = create_abstr(&item_struct)?;

    if has_derived_partial_eq && has_derived_eq {
        // add phi and meta-eq implementations
        let phi_impl = phi_impl(&item_struct)?;
        let meta_eq_impl = meta_eq_impl(&item_struct);

        Ok(vec![
            Item::Struct(item_struct),
            abstr_impl,
            meta_eq_impl,
            phi_impl,
        ])
    } else {
        Ok(vec![Item::Struct(item_struct), abstr_impl])
    }
}

fn create_abstr(item_struct: &ItemStruct) -> Result<Item, Error> {
    let span = item_struct.span();

    let mut concr_segments = Punctuated::new();
    concr_segments.push(create_path_segment(Ident::new("super", span)));
    concr_segments.push(create_path_segment(item_struct.ident.clone()));
    let concr_path = Path {
        leading_colon: None,
        segments: concr_segments,
    };
    let concr_ty = create_type_path(concr_path);
    let abstr_path =
        create_path_with_last_generic_type(path!(::mck::abstr::Abstr), concr_ty.clone());

    let from_concrete_fn = ImplItem::Fn(from_concrete_fn(item_struct, concr_ty)?);

    Ok(Item::Impl(ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Token![impl](span),
        generics: Generics::default(),
        trait_: Some((None, abstr_path, Token![for](span))),
        self_ty: Box::new(create_type_path(create_path_from_ident(
            item_struct.ident.clone(),
        ))),
        brace_token: Default::default(),
        items: vec![from_concrete_fn],
    }))
}
