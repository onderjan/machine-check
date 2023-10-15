use syn::{parse_quote, ImplItem, Item, ItemImpl, Type};

use crate::machine::util::{
    create_ident, create_impl_item_type, create_type_path,
    path_rule::{self},
    scheme::ConversionScheme,
};

use self::convert::MarkConverter;

use super::mark_path_rules;
use anyhow::anyhow;
use quote::quote;

mod convert;

pub fn apply(mark_file_items: &mut Vec<Item>, i: &ItemImpl) -> Result<(), anyhow::Error> {
    let mut transcribed = transcribe_item_impl(i)?;
    if let Some(trait_) = &mut transcribed.trait_ {
        path_rule::apply_to_path(&mut trait_.1, mark_path_rules())?;
    }

    if let Some((None, trait_path, _)) = &i.trait_ {
        if trait_path.leading_colon.is_some() {
            let mut iter = trait_path.segments.iter();
            if let Some(crate_seg) = iter.next() {
                if let Some(flavour_seg) = iter.next() {
                    if let Some(type_seg) = iter.next() {
                        if crate_seg.ident == "mck"
                            && flavour_seg.ident == "abstr"
                            && (type_seg.ident == "Input"
                                || type_seg.ident == "State"
                                || type_seg.ident == "Machine")
                        {
                            let s_ty = i.self_ty.as_ref();
                            // add abstract type
                            let type_ident = create_ident("Abstract");
                            let type_assign = create_type_path(parse_quote!(super::#s_ty));
                            transcribed.items.push(ImplItem::Type(create_impl_item_type(
                                type_ident,
                                type_assign,
                            )));
                        }
                    }
                }
            }
        }
    }

    mark_file_items.push(Item::Impl(transcribed));
    Ok(())
}

pub fn transcribe_item_impl(i: &ItemImpl) -> anyhow::Result<ItemImpl> {
    let mut i = i.clone();
    let mut items = Vec::<ImplItem>::new();

    let self_ty = i.self_ty.as_ref();

    let Type::Path(self_ty) = self_ty else {
        return Err(anyhow!("Non-path impl type '{}' not supported", quote!(#self_ty)));
    };

    let Some(self_ty_ident) = self_ty.path.get_ident() else {
        return Err(anyhow!("Non-ident impl type '{}' not supported", quote!(#self_ty)));
    };
    let self_name = self_ty_ident.to_string();

    let mut converter = MarkConverter {
        abstract_scheme: ConversionScheme::new(
            String::from("__mck_"),
            String::from("abstr"),
            self_name.clone(),
            true,
        ),
        mark_scheme: ConversionScheme::new(
            String::from("__mck_"),
            String::from("refin"),
            self_name,
            false,
        ),
    };

    for item in &i.items {
        match item {
            ImplItem::Fn(item_fn) => {
                let mark_fn = converter.transcribe_impl_item_fn(item_fn)?;
                items.push(ImplItem::Fn(mark_fn));
            }
            ImplItem::Type(item_type) => {
                // just clone for now
                items.push(ImplItem::Type(item_type.clone()));
            }
            _ => return Err(anyhow!("Impl item type {:?} not supported", item)),
        }
    }

    i.items = items;
    Ok(i)
}
