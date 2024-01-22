mod convert_mutable;
mod infer_types;
mod normalize_expressions;
mod normalize_scope;

use syn::Item;

use crate::{MachineDescription, MachineError};

pub(crate) fn create_concrete_machine(
    mut items: Vec<Item>,
) -> Result<MachineDescription, MachineError> {
    normalize_scope::normalize_scope(&mut items)?;
    convert_mutable::convert_mutable(&mut items)?;
    normalize_expressions::normalize_expressions(&mut items)?;
    infer_types::infer_types(&mut items)?;

    Ok(MachineDescription { items })
}
