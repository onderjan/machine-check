use core::panic;

use item::{fold_item_impl, fold_item_struct};
use syn::Item;

use crate::wir::{WDescription, YTac};

use super::error::DescriptionErrors;

pub mod impl_item_fn;
pub mod item;
pub mod path;
pub mod ty;

pub fn from_syn(
    item_iter: impl Iterator<Item = Item>,
) -> Result<WDescription<YTac>, DescriptionErrors> {
    let mut structs = Vec::new();
    let mut impls = Vec::new();
    for item in item_iter {
        match item {
            Item::Struct(item) => structs.push(fold_item_struct(item)),
            Item::Impl(item) => impls.push(fold_item_impl(item)),
            _ => panic!("Unexpected type of item: {:?}", item),
        }
    }

    let structs = DescriptionErrors::flat_result(structs)?;
    let impls = DescriptionErrors::flat_result(impls)?;

    Ok(WDescription { structs, impls })
}
