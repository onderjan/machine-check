mod item_impl;
mod item_struct;
mod rules;

use syn::Item;

use crate::{
    support::manipulate::{self, ManipulateKind},
    Description, ErrorType,
};

use self::{
    item_impl::{preprocess_item_impl, process_item_impl},
    item_struct::process_item_struct,
    rules::path_rules,
};

use super::MachineError;

pub(crate) fn create_abstract_description(
    description: &Description,
) -> Result<Description, MachineError> {
    let mut abstract_description = description.clone();
    // apply transcription to types using path rule transcriptor
    match path_rules().apply_to_items(&mut abstract_description.items) {
        Ok(()) => {}
        Err(err) => {
            return Err(MachineError::new(
                ErrorType::ForwardConversionError(String::from("Conversion not known")),
                err.0,
            ));
        }
    }

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
                processed_items.extend(process_item_impl(item_impl, &machine_types)?);
            }
            syn::Item::Struct(item_struct) => {
                processed_items.extend(process_item_struct(item_struct)?);
            }
            _ => panic!("Unexpected item type"),
        }
    }
    abstract_description.items = processed_items;

    // add field-manipulate to items
    manipulate::apply_to_items(&mut abstract_description.items, ManipulateKind::Abstr);

    Ok(abstract_description)
}
