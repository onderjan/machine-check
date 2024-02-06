use syn::{Item, ItemStruct};

use crate::{support::meta_eq::meta_eq_impl, MachineError};

use self::{meta::meta_impl, refine::refine_impl};

use super::SpecialTrait;

mod meta;
mod refine;

pub(super) fn add_special_impls(
    special_trait: SpecialTrait,
    refinement_items: &mut Vec<Item>,
    item_struct: &ItemStruct,
) -> Result<(), MachineError> {
    match special_trait {
        SpecialTrait::Input | SpecialTrait::State => {
            // add Meta and Refine implementations
            refinement_items.push(Item::Impl(meta_impl(item_struct)?));
            refinement_items.push(refine_impl(item_struct)?);
            refinement_items.push(meta_eq_impl(item_struct));
        }

        SpecialTrait::Machine => {
            // add Refine implementation
            refinement_items.push(refine_impl(item_struct)?);
        }
    }

    Ok(())
}
