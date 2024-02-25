mod convert_to_ssa;
mod convert_to_tac;
mod convert_types;
mod expand_macros;
mod infer_types;
mod normalize_constructs;
mod normalize_scope;

use syn::Item;

use crate::{support::block_converter::TemporaryManager, MachineDescription, MachineError};

pub(crate) fn create_concrete_machine(
    mut items: Vec<Item>,
) -> Result<MachineDescription, MachineError> {
    let mut temporary_manager = TemporaryManager::new();

    expand_macros::expand_macros(&mut items)?;
    normalize_constructs::normalize_constructs(&mut items)?;
    normalize_scope::normalize_scope(&mut items);
    convert_to_tac::convert_to_tac(&mut items, &mut temporary_manager)?;
    convert_to_ssa::convert_to_ssa(&mut items)?;
    infer_types::infer_types(&mut items)?;
    convert_types::convert_types(&mut items)?;

    Ok(MachineDescription { items })
}
