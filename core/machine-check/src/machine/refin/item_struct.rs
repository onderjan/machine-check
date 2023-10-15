use syn::{Item, ItemStruct};

use crate::machine::Error;

use self::{meta::meta_impl, refinable::refinable_impl, refine::refine_impl};

use super::SpecialTrait;

mod meta;
mod refinable;
mod refine;

pub(super) fn add_special_impls(
    special_trait: SpecialTrait,
    refinement_items: &mut Vec<Item>,
    item_struct: &ItemStruct,
) -> Result<(), Error> {
    match special_trait {
        SpecialTrait::Input | SpecialTrait::State => {
            // add Meta and Refinable implementations
            refinement_items.push(Item::Impl(meta_impl(item_struct)?));
            refinement_items.push(Item::Impl(refinable_impl(item_struct)?));
            // add Refine implementation
            refinement_items.push(refine_impl(item_struct)?);
        }

        SpecialTrait::Machine => {
            // do nothing for now
        }
    }

    Ok(())
}
