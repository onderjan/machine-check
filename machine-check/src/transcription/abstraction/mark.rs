use proc_macro2::Span;
use syn::{
    token::Brace, Ident, ImplItem, Item, ItemImpl, ItemMod, ItemStruct, Path, PathSegment, Type,
    TypePath,
};

use crate::transcription::util::path_rule::{self, PathRule, PathRuleSegment};

use anyhow::anyhow;

use quote::quote;

use self::mark_fn::transcribe_item_impl;

mod mark_fn;
mod mark_ident;
mod mark_stmt;
mod mark_type_path;

pub fn apply(file: &mut syn::File) -> anyhow::Result<()> {
    // the mark will be in a new module under the abstract

    // create items to add to the module
    let mut mark_file_items = Vec::<Item>::new();
    for item in &file.items {
        let transcribed_item = match item {
            Item::Struct(s) => Item::Struct(transcribe_item_struct(s)?),
            Item::Impl(i) => Item::Impl(transcribe_item_impl(i)?),
            _ => {
                return Err(anyhow::anyhow!("Item type {:?} not supported", item));
            }
        };
        mark_file_items.push(transcribed_item);
    }
    // create new module at the end of the file that will contain the mark

    let mod_mark = Item::Mod(ItemMod {
        attrs: vec![],
        vis: syn::Visibility::Public(Default::default()),
        unsafety: None,
        mod_token: Default::default(),
        ident: Ident::new("mark", Span::call_site()),
        content: Some((Brace::default(), mark_file_items)),
        semi: None,
    });
    file.items.push(mod_mark);
    Ok(())
}

fn transcribe_item_struct(s: &ItemStruct) -> anyhow::Result<ItemStruct> {
    let mut s = s.clone();
    path_rule::apply_to_item_struct(&mut s, path_rules())?;
    Ok(s)
}

fn path_rules() -> Vec<PathRule> {
    vec![
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Ident(String::from("mck")),
                PathRuleSegment::Convert(
                    String::from("ThreeValuedArray"),
                    String::from("MarkArray"),
                ),
            ],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Ident(String::from("mck")),
                PathRuleSegment::Convert(
                    String::from("ThreeValuedBitvector"),
                    String::from("MarkBitvector"),
                ),
            ],
        },
    ]
}
