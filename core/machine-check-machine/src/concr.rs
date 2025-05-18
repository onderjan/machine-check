mod item_impl;

use syn::Item;

use crate::Error;

use self::item_impl::process_item_impl;

pub fn process_items(items: &mut Vec<Item>, panic_messages: &[String]) -> Result<(), Error> {
    let mut added_items = Vec::new();
    for item in items.iter_mut() {
        match item {
            syn::Item::Impl(ref mut item_impl) => {
                // add concrete traits for inputs, states, and machines
                added_items.extend(process_item_impl(item_impl, panic_messages)?);
            }
            syn::Item::Struct(_) | syn::Item::Use(_) => {
                // do nothing
            }
            _ => panic!("Unexpected item type"),
        }
    }
    items.extend(added_items);
    Ok(())
}
