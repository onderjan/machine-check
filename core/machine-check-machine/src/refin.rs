use std::collections::HashMap;

use syn::{Ident, Item, Type};

use crate::{util::path_matches_global_names, MachineDescription};

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
    println!("Refining abstract machine");

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
    println!("Refined abstract machine");

    Ok(refinement_machine)
}

fn is_skipped_impl(item_impl: &syn::ItemImpl) -> bool {
    let Some((_, path, _)) = &item_impl.trait_ else {
        return false;
    };
    path_matches_global_names(path, &["mck", "misc", "FieldManipulate"])
        || path_matches_global_names(path, &["mck", "misc", "MetaEq"])
}
