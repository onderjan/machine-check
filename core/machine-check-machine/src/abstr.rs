mod item_impl;
mod item_struct;

use syn::Item;

use crate::{
    support::manipulate::{self, ManipulateKind},
    wir::{IntoSyn, WDescription, YConverted},
    Description,
};

use self::{
    item_impl::{preprocess_item_impl, process_item_impl},
    item_struct::process_item_struct,
};

use super::Error;

pub(crate) fn create_abstract_description(
    description: &WDescription<YConverted>,
) -> Result<Description, Error> {
    let items = description.clone().into_syn().items;
    let mut abstract_description = Description { items };

    let mut machine_types = Vec::new();
    let mut processed_items = Vec::new();

    for item in abstract_description.items.iter() {
        if let Item::Impl(item_impl) = item {
            if let Some(ty) = preprocess_item_impl(item_impl)? {
                machine_types.push(ty);
            }
        }
    }

    for item in abstract_description.items.drain(..) {
        match item {
            syn::Item::Impl(item_impl) => {
                let item_impls = process_item_impl(item_impl, &machine_types)?;
                processed_items.extend(item_impls.into_iter().map(Item::Impl));
            }
            syn::Item::Struct(item_struct) => {
                let (item_struct, other_impls) = process_item_struct(item_struct)?;
                processed_items.push(Item::Struct(item_struct));
                processed_items.extend(other_impls.into_iter().map(Item::Impl));
            }
            _ => panic!("Unexpected item type"),
        }
    }
    abstract_description.items = processed_items;

    // add field-manipulate to items
    manipulate::apply_to_items(&mut abstract_description.items, ManipulateKind::Forward);

    Ok(abstract_description)
}
