use item::{fold_item_impl, fold_item_struct};
use syn::{spanned::Spanned, Item};

use crate::wir::{WDescription, YTac};

use super::error::{DescriptionError, DescriptionErrors};

pub mod impl_item_fn;
pub mod item;
pub mod path;
pub mod ty;

pub fn from_syn(
    item_iter: impl Iterator<Item = Item>,
) -> Result<WDescription<YTac>, DescriptionErrors> {
    let mut structs = Vec::new();
    let mut impls = Vec::new();
    let mut errors = Vec::new();
    for item in item_iter {
        match item {
            Item::Struct(item) => structs.push(fold_item_struct(item)),
            Item::Impl(item) => impls.push(fold_item_impl(item)),
            _ => errors.push(DescriptionError::unsupported_construct(
                "Item kind",
                item.span(),
            )),
        }
    }

    let structs = DescriptionErrors::flat_result(structs);
    let impls = DescriptionErrors::flat_result(impls);
    let (structs, impls) = match DescriptionErrors::combine(structs, impls) {
        Ok(ok) => ok,
        Err(mut err) => {
            err.extend(errors);
            return Err(err);
        }
    };

    DescriptionErrors::iter_to_result(errors)?;

    Ok(WDescription { structs, impls })
}
