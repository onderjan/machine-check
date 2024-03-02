use crate::{support::block_convert::TemporaryManager, MachineError};
use syn::Item;

use self::{demacroed::convert_demacroed_items, typed::convert_typed_item};

mod demacroed;
mod typed;

pub fn convert_panic_demacroed(
    items: &mut [Item],
    temporary_manager: &mut TemporaryManager,
) -> Result<(), MachineError> {
    convert_demacroed_items(items, temporary_manager)
}

pub fn convert_panic_typed(items: &mut [Item]) -> Result<(), MachineError> {
    for item in items.iter_mut() {
        convert_typed_item(item)?;
    }
    Ok(())
}
