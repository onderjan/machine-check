use std::collections::HashMap;

use syn::{Ident, Item, Type};

use crate::MachineDescription;

use super::{
    support::{
        field_manipulate,
        special_trait::{special_trait_impl, SpecialTrait},
    },
    MachineError,
};

mod item_impl;
mod item_struct;
mod rules;

pub(crate) fn create_refinement_machine(
    abstract_machine: &MachineDescription,
) -> Result<MachineDescription, MachineError> {
    // the refinement machine will be in a new module at the end of the file

    // create items to add to the module
    let mut result_items = Vec::<Item>::new();
    let mut ident_special_traits = HashMap::<Ident, SpecialTrait>::new();

    // first pass
    for item in &abstract_machine.items {
        match item {
            Item::Struct(item_struct) => {
                // apply path rules and push struct
                let mut refin_struct = item_struct.clone();
                rules::refinement_normal().apply_to_item_struct(&mut refin_struct)?;
                result_items.push(Item::Struct(refin_struct));
            }
            Item::Impl(item_impl) => {
                // look for special traits
                item_impl::apply(&mut result_items, item_impl)?;
                if let Some(special_trait) = special_trait_impl(item_impl, "abstr") {
                    if let Type::Path(ty) = item_impl.self_ty.as_ref() {
                        if let Some(ident) = ty.path.get_ident() {
                            ident_special_traits.insert(ident.clone(), special_trait);
                        }
                    }
                };
            }
            _ => {
                return Err(MachineError(format!("Item type {:?} not supported", item)));
            }
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
    field_manipulate::apply_to_items(&mut result_items, "refin")?;

    let refinement_machine = MachineDescription {
        items: result_items,
    };

    Ok(refinement_machine)
}
