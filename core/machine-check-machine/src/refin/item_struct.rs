use syn::{Item, ItemStruct};

use crate::{
    support::meta_eq::meta_eq_impl, util::create_path_from_ident, ErrorType, MachineError,
};

use self::{meta::meta_impl, refine::refine_impl};

use super::{rules, SpecialTrait};

mod meta;
mod refine;

pub(super) fn add_special_impls(
    special_trait: SpecialTrait,
    refinement_items: &mut Vec<Item>,
    item_struct: &ItemStruct,
) -> Result<(), MachineError> {
    let abstr_type_path = match rules::abstract_type()
        .convert_path(create_path_from_ident(item_struct.ident.clone()))
    {
        Ok(ok) => ok,
        Err(err) => {
            return Err(MachineError::new(
                ErrorType::BackwardConversionError(String::from("Unable to convert struct ident")),
                err.0,
            ));
        }
    };

    match special_trait {
        SpecialTrait::Input | SpecialTrait::State => {
            // add Meta and Refine implementations
            refinement_items.push(Item::Impl(meta_impl(item_struct, &abstr_type_path)?));
            refinement_items.push(refine_impl(item_struct, &abstr_type_path)?);
            refinement_items.push(meta_eq_impl(item_struct));
        }

        SpecialTrait::Machine => {
            // add Refine implementation
            refinement_items.push(refine_impl(item_struct, &abstr_type_path)?);
        }
    }

    Ok(())
}
