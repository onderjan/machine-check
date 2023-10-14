use proc_macro2::Span;
use syn::{token::Brace, Ident, Item, ItemMod};

use crate::machine::util::path_rule::{PathRule, PathRuleSegment};

use self::{refin_impl::apply_to_impl, refin_struct::apply_to_struct};

mod refin_fn;
mod refin_impl;
mod refin_stmt;
mod refin_struct;

pub fn apply(file: &mut syn::File) -> anyhow::Result<()> {
    // the mark will be in a new module under the abstract

    // create items to add to the module
    let mut mark_file_items = Vec::<Item>::new();
    for item in &file.items {
        match item {
            Item::Struct(s) => apply_to_struct(&mut mark_file_items, s)?,
            Item::Impl(i) => apply_to_impl(&mut mark_file_items, i)?,
            _ => {
                return Err(anyhow::anyhow!("Item type {:?} not supported", item));
            }
        };
    }
    // create new module at the end of the file that will contain the mark

    let mod_mark = Item::Mod(ItemMod {
        attrs: vec![],
        vis: syn::Visibility::Public(Default::default()),
        unsafety: None,
        mod_token: Default::default(),
        ident: Ident::new("refin", Span::call_site()),
        content: Some((Brace::default(), mark_file_items)),
        semi: None,
    });
    file.items.push(mod_mark);
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
