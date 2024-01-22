mod convert_to_ssa;
mod infer_types;
mod normalize_expressions;
mod normalize_scope;

use syn::Item;

use crate::{MachineDescription, MachineError};

pub(crate) fn create_concrete_machine(
    mut items: Vec<Item>,
) -> Result<MachineDescription, MachineError> {
    normalize_scope::normalize_scope(&mut items)?;
    convert_to_ssa::convert_to_ssa(&mut items)?;
    normalize_expressions::normalize_expressions(&mut items)?;
    infer_types::infer_types(&mut items)?;

    Ok(MachineDescription { items })
}
