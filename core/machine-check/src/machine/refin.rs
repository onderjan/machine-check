use std::collections::HashMap;

use proc_macro2::Span;
use syn::{Ident, Item, Type};

use super::util::create_item_mod;

mod item_impl;
mod item_struct;
mod rules;

pub fn apply(abstract_machine_file: &mut syn::File) -> anyhow::Result<()> {
    // the refinement machine will be in a new module at the end of the file

    // create items to add to the module
    let mut refinement_items = Vec::<Item>::new();
    let mut ident_special_traits = HashMap::<Ident, SpecialTrait>::new();

    // first pass
    for item in &abstract_machine_file.items {
        match item {
            Item::Struct(s) => {
                // apply path rules and push struct
                let mut refin_struct = s.clone();
                rules::refinement_normal().apply_to_item_struct(&mut refin_struct)?;
                refinement_items.push(Item::Struct(refin_struct));
            }
            Item::Impl(i) => {
                // look for special traits
                let special_trait = item_impl::apply(&mut refinement_items, i)?;
                if let Some(special_trait) = special_trait {
                    if let Type::Path(ty) = i.self_ty.as_ref() {
                        if let Some(ident) = ty.path.get_ident() {
                            ident_special_traits.insert(ident.clone(), special_trait);
                        }
                    }
                };
            }
            _ => {
                return Err(anyhow::anyhow!("Item type {:?} not supported", item));
            }
        };
    }
    // second pass, add special impls for special traits
    for item in &abstract_machine_file.items {
        if let Item::Struct(s) = item {
            if let Some(special_trait) = ident_special_traits.remove(&s.ident) {
                item_struct::add_special_impls(special_trait, &mut refinement_items, s)?;
            }
        }
    }

    // create new module at the end of the file that will contain the refinement
    let refinement_module = Item::Mod(create_item_mod(
        syn::Visibility::Public(Default::default()),
        Ident::new("refin", Span::call_site()),
        refinement_items,
    ));
    abstract_machine_file.items.push(refinement_module);
    Ok(())
}

enum SpecialTrait {
    Machine,
    Input,
    State,
}
