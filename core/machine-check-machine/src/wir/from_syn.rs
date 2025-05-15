use core::panic;

use item::{fold_item_impl, fold_item_struct};
use syn::Item;

use crate::MachineErrors;

use super::{WDescription, YTac};

pub mod impl_item_fn;
pub mod item;
pub mod path;
pub mod ty;

impl WDescription<YTac> {
    pub fn from_syn(
        item_iter: impl Iterator<Item = Item>,
    ) -> Result<WDescription<YTac>, MachineErrors> {
        let mut structs = Vec::new();
        let mut impls = Vec::new();
        for item in item_iter {
            match item {
                Item::Struct(item) => structs.push(fold_item_struct(item)),
                Item::Impl(item) => impls.push(fold_item_impl(item)),
                _ => panic!("Unexpected type of item: {:?}", item),
            }
        }

        let structs = MachineErrors::flat_result(structs)?;
        let impls = MachineErrors::flat_result(impls)?;

        Ok(WDescription { structs, impls })
    }
}
