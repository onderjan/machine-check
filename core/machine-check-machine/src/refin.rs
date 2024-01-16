use std::collections::HashMap;

use proc_macro2::Span;
use syn::{Ident, Item, Type};

use crate::Machine;

use super::{
    support::{
        field_manipulate,
        special_trait::{special_trait_impl, SpecialTrait},
    },
    util::create_item_mod,
    MachineError,
};

mod item_impl;
mod item_struct;
mod rules;

pub(crate) fn apply(abstract_machine: &mut Machine) -> Result<(), MachineError> {
    // the refinement machine will be in a new module at the end of the file

    // create items to add to the module
    let mut refinement_items = Vec::<Item>::new();
    let mut ident_special_traits = HashMap::<Ident, SpecialTrait>::new();

    // first pass
    for item in &abstract_machine.items {
        match item {
            Item::Struct(item_struct) => {
                // apply path rules and push struct
                let mut refin_struct = item_struct.clone();
                rules::refinement_normal().apply_to_item_struct(&mut refin_struct)?;
                refinement_items.push(Item::Struct(refin_struct));
            }
            Item::Impl(item_impl) => {
                // look for special traits
                item_impl::apply(&mut refinement_items, item_impl)?;
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
                item_struct::add_special_impls(special_trait, &mut refinement_items, s)?;
            }
        }
    }

    // add field manipulate
    field_manipulate::apply_to_items(&mut refinement_items, "refin")?;
    field_manipulate::apply_to_items(&mut abstract_machine.items, "abstr")?;

    // create new module at the end of the file that will contain the refinement
    let refinement_module = Item::Mod(create_item_mod(
        syn::Visibility::Public(Default::default()),
        Ident::new("refin", Span::call_site()),
        refinement_items,
    ));
    abstract_machine.items.push(refinement_module);
    Ok(())
}
