use proc_macro2::Span;
use syn::{token::Brace, Ident, Item, ItemMod};

use self::{mark_impl::transcribe_impl, mark_type_path::TypePathVisitor};

mod mark_ident;
mod mark_impl;
mod mark_stmt;
mod mark_type_path;

pub fn transcribe(file: &mut syn::File) -> anyhow::Result<()> {
    let mut mark_file_items = Vec::<Item>::new();
    for item in &file.items {
        match item {
            Item::Struct(s) => {
                let mut mark_s = s.clone();
                TypePathVisitor::new().visit_struct(&mut mark_s);
                mark_file_items.push(Item::Struct(mark_s));
            }
            Item::Impl(i) => mark_file_items.push(Item::Impl(transcribe_impl(i)?)),
            _ => {
                return Err(anyhow::anyhow!("Item type {:?} not supported", item));
            }
        }
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
