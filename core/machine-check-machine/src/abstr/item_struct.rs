mod phi;
use quote::{quote, ToTokens};
use syn::{parse_quote, punctuated::Punctuated, Item, ItemStruct, Path, Token};

use crate::{
    abstr::item_struct::phi::phi_impl, support::meta_eq::meta_eq_impl,
    util::generate_derive_attribute, MachineError,
};

pub fn process_impl_item_struct(mut item_struct: ItemStruct) -> Result<Vec<Item>, MachineError> {
    let mut has_derived_eq = false;
    let mut has_derived_partial_eq = false;
    // remove derives of PartialEq and Eq
    // only if they were derived, we can derive corresponding abstract traits
    for attr in item_struct.attrs.iter_mut() {
        if let syn::Meta::List(meta_list) = &mut attr.meta {
            if meta_list.path.is_ident("derive") {
                let tokens = &meta_list.tokens;
                let punctuated: Punctuated<Path, Token![,]> = parse_quote!(#tokens);
                let mut processed_punctuated: Punctuated<Path, Token![,]> = Punctuated::new();
                for derive in punctuated {
                    // TODO: resolve paths
                    if derive.is_ident("PartialEq") {
                        has_derived_partial_eq = true;
                    } else if derive.is_ident("Eq") {
                        has_derived_eq = true;
                    } else {
                        processed_punctuated.push(derive);
                    }
                }
                meta_list.tokens = processed_punctuated.to_token_stream();
            }
        }
    }

    if has_derived_partial_eq && has_derived_eq {
        // add trait implementations
        let phi_impl = phi_impl(&item_struct)?;

        let meta_eq_impl = meta_eq_impl(&item_struct);

        Ok(vec![Item::Struct(item_struct), meta_eq_impl, phi_impl])
    } else {
        Ok(vec![Item::Struct(item_struct)])
    }
}
