mod convert_indexing;
mod convert_panic;
mod convert_to_ssa;
mod convert_to_tac;
mod convert_types;
mod expand_macros;
mod infer_types;
mod normalize_constructs;
mod normalize_scope;
mod resolve_use;

use syn::Item;

use crate::{support::block_convert::TemporaryManager, MachineDescription, MachineError};

pub(crate) fn create_concrete_machine(
    mut items: Vec<Item>,
) -> Result<MachineDescription, MachineError> {
    let mut temporary_manager = TemporaryManager::new();

    resolve_use::resolve_use(&mut items)?;
    expand_macros::expand_macros(&mut items)?;
    resolve_use::resolve_use(&mut items)?;
    resolve_use::remove_use(&mut items)?;
    normalize_constructs::normalize_constructs(&mut items)?;
    convert_panic::convert_panic_demacroed(&mut items, &mut temporary_manager)?;
    normalize_scope::normalize_scope(&mut items);
    convert_to_tac::convert_to_tac(&mut items, &mut temporary_manager);
    convert_indexing::convert_indexing(&mut items, &mut temporary_manager)?;
    convert_to_ssa::convert_to_ssa(&mut items)?;
    infer_types::infer_types(&mut items)?;
    convert_types::convert_types(&mut items)?;
    convert_panic::convert_panic_typed(&mut items)?;

    Ok(MachineDescription { items })
}
