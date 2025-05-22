use syn::{Item, ItemStruct};

use crate::{support::meta_eq::meta_eq_impl, util::create_path_from_ident, BackwardError};

use self::{meta::meta_impl, refine::refine_impl};

use super::{rules, SpecialTrait};

mod meta;
mod refine;

pub(super) fn add_special_impls(
    special_trait: SpecialTrait,
    refinement_items: &mut Vec<Item>,
    item_struct: &ItemStruct,
) -> Result<(), BackwardError> {
    let abstr_type_path = rules::abstract_rules()
        .convert_type_path(create_path_from_ident(item_struct.ident.clone()))?;

    match special_trait {
        SpecialTrait::Input | SpecialTrait::State => {
            // add Meta and Refine implementations
            refinement_items.push(Item::Impl(meta_impl(item_struct, &abstr_type_path)?));
            refinement_items.push(refine_impl(item_struct, &abstr_type_path)?);
            refinement_items.push(Item::Impl(meta_eq_impl(item_struct)));
        }

        SpecialTrait::Machine => {
            // add Refine implementation
            refinement_items.push(refine_impl(item_struct, &abstr_type_path)?);
        }
    }

    Ok(())
}
