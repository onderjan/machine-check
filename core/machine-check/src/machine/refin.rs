use proc_macro2::Span;
use syn::{Ident, Item};

use super::util::create_item_mod;

mod item_impl;
mod item_struct;
mod rules;

pub fn apply(abstract_machine_file: &mut syn::File) -> anyhow::Result<()> {
    // the refinement machine will be in a new module at the end of the file

    // create items to add to the module
    let mut refinement_items = Vec::<Item>::new();
    for item in &abstract_machine_file.items {
        match item {
            Item::Struct(s) => item_struct::apply(&mut refinement_items, s)?,
            Item::Impl(i) => item_impl::apply(&mut refinement_items, i)?,
            _ => {
                return Err(anyhow::anyhow!("Item type {:?} not supported", item));
            }
        };
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
