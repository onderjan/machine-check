use anyhow::anyhow;
use syn::{ImplItem, ItemImpl};

use super::mark_fn::transcribe_impl_item_fn;

pub fn transcribe_impl(i: &ItemImpl) -> anyhow::Result<ItemImpl> {
    let mut mark_i = i.clone();

    let mut mark_items = Vec::<ImplItem>::new();

    for item in mark_i.items {
        let ImplItem::Fn(item_fn) = item else {
            return Err(anyhow!("Impl item type {:?} not supported", item));
        };
        let mut mark_fn = item_fn.clone();
        transcribe_impl_item_fn(&mut mark_fn, mark_i.self_ty.as_ref())?;

        mark_items.push(ImplItem::Fn(mark_fn));
    }

    mark_i.items = mark_items;

    Ok(mark_i)
}
