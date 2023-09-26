use proc_macro2::Span;
use syn::{token::Brace, Ident, ImplItem, Item, ItemImpl, ItemMod, ItemStruct};

use crate::transcription::util::path_rule::{self, PathRule, PathRuleSegment};

use anyhow::anyhow;

use self::mark_fn::transcribe_impl_item_fn;

mod mark_fn;
mod mark_ident;
mod mark_stmt;
mod mark_type_path;

pub fn transcribe(file: &mut syn::File) -> anyhow::Result<()> {
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
    path_rule::transcribe_item_struct(&mut s, path_rules())?;
    Ok(s)
}

fn transcribe_item_impl(i: &ItemImpl) -> anyhow::Result<ItemImpl> {
    let mut i = i.clone();
    let mut items = Vec::<ImplItem>::new();

    for item in i.items {
        if let ImplItem::Fn(item_fn) = item {
            let mut mark_fn = item_fn.clone();
            transcribe_impl_item_fn(&mut mark_fn, i.self_ty.as_ref())?;
            items.push(ImplItem::Fn(mark_fn));
        } else {
            return Err(anyhow!("Impl item type {:?} not supported", item));
        };
    }

    i.items = items;
    Ok(i)
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
