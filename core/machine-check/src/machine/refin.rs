use proc_macro2::Span;
use syn::{Ident, Item};

use crate::machine::util::path_rule::{PathRule, PathRuleSegment};

use self::{refin_impl::apply_to_impl, refin_struct::apply_to_struct};

use super::util::create_item_mod;

mod refin_fn;
mod refin_impl;
mod refin_stmt;
mod refin_struct;

pub fn apply(abstract_machine_file: &mut syn::File) -> anyhow::Result<()> {
    // the refinement machine will be in a new module at the end of the file

    // create items to add to the module
    let mut mark_file_items = Vec::<Item>::new();
    for item in &abstract_machine_file.items {
        match item {
            Item::Struct(s) => apply_to_struct(&mut mark_file_items, s)?,
            Item::Impl(i) => apply_to_impl(&mut mark_file_items, i)?,
            _ => {
                return Err(anyhow::anyhow!("Item type {:?} not supported", item));
            }
        };
    }
    // create new module at the end of the file that will contain the refinement
    let mod_mark = Item::Mod(create_item_mod(
        syn::Visibility::Public(Default::default()),
        Ident::new("refin", Span::call_site()),
        mark_file_items,
    ));
    abstract_machine_file.items.push(mod_mark);
    Ok(())
}

pub fn mark_path_rules() -> Vec<PathRule> {
    vec![PathRule {
        has_leading_colon: true,
        segments: vec![
            PathRuleSegment::Ident(String::from("mck")),
            PathRuleSegment::Convert(String::from("abstr"), String::from("refin")),
            PathRuleSegment::Wildcard,
        ],
    }]
}
