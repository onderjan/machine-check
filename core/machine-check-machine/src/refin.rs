use core::panic;
use std::collections::HashMap;

use syn::{Ident, Item, Type};

use crate::{
    support::manipulate::{self, ManipulateKind},
    util::path_matches_global_names,
    BackwardError, MachineDescription,
};

use super::support::special_trait::{special_trait_impl, SpecialTrait};

mod item_impl;
mod item_struct;
mod rules;
mod util;

pub(crate) fn create_refinement_machine(
    abstract_machine: &MachineDescription,
) -> Result<MachineDescription, BackwardError> {
    // create items to add to the module
    let mut result_items = Vec::<Item>::new();
    let mut ident_special_traits = HashMap::<Ident, SpecialTrait>::new();

    // first pass
    for item in &abstract_machine.items {
        match item {
            Item::Struct(item_struct) => {
                // apply path rules and push struct
                let mut refin_struct = item_struct.clone();
                rules::refinement_rules().apply_to_item_struct(&mut refin_struct)?;
                result_items.push(Item::Struct(refin_struct));
            }
            Item::Impl(item_impl) => {
                // skip if it is field-manipulate, we will add it at the end
                if !is_skipped_impl(item_impl) {
                    // apply conversion
                    item_impl::apply(&mut result_items, item_impl)?;
                    // look for special traits
                    if let Some(special_trait) = special_trait_impl(item_impl, "abstr") {
                        if let Type::Path(ty) = item_impl.self_ty.as_ref() {
                            if let Some(ident) = ty.path.get_ident() {
                                ident_special_traits.insert(ident.clone(), special_trait);
                            }
                        }
                    };
                }
            }
            _ => panic!("Unexpected item type"),
        };
    }
    // second pass, add special impls for special traits
    for item in &abstract_machine.items {
        if let Item::Struct(s) = item {
            if let Some(special_trait) = ident_special_traits.remove(&s.ident) {
                item_struct::add_special_impls(special_trait, &mut result_items, s)?;
            }
        }
    }

    // add field manipulate
    manipulate::apply_to_items(&mut result_items, ManipulateKind::Refin);

    let refinement_machine = MachineDescription {
        items: result_items,
        panic_messages: abstract_machine.panic_messages.clone(),
    };

    Ok(refinement_machine)
}

fn is_skipped_impl(item_impl: &syn::ItemImpl) -> bool {
    let Some((_, path, _)) = &item_impl.trait_ else {
        return false;
    };
    path_matches_global_names(path, &["mck", "abstr", "Manipulatable"])
        || path_matches_global_names(path, &["mck", "abstr", "Phi"])
        || path_matches_global_names(path, &["mck", "abstr", "Abstr"])
        || path_matches_global_names(path, &["mck", "misc", "MetaEq"])
}
