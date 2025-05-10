use crate::{support::block_convert::TemporaryManager, MachineError};
use syn::Item;

use self::demacroed::convert_demacroed_items;

mod demacroed;

pub fn convert_panic_demacroed(
    items: &mut [Item],
    temporary_manager: &mut TemporaryManager,
) -> Result<(), MachineError> {
    convert_demacroed_items(items, temporary_manager)
}
