use core::panic;

use item::{fold_item_impl, fold_item_struct};
use syn::Item;

use super::{WDescription, YTac};

pub mod expr;
pub mod impl_item;
pub mod item;
pub mod path;
pub mod stmt;
pub mod ty;

impl WDescription<YTac> {
    pub fn from_syn(item_iter: impl Iterator<Item = Item>) -> WDescription<YTac> {
        let mut structs = Vec::new();
        let mut impls = Vec::new();
        for item in item_iter {
            match item {
                Item::Struct(item) => structs.push(fold_item_struct(item)),
                Item::Impl(item) => impls.push(fold_item_impl(item)),
                _ => panic!("Unexpected type of item: {:?}", item),
            }
        }

        WDescription { structs, impls }
    }
}
