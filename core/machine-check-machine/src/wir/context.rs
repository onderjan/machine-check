use syn::Type;

use crate::wir::{WTypeDef, WTypeId};

pub struct WContext {
    types: Vec<WTypeDef>,
}

pub struct RequiresInferenceError;

impl WContext {
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }

    pub fn get_noninferred_type(&mut self, ty: &Type) -> Result<WTypeId, RequiresInferenceError> {
        let id = WTypeId(self.types.len());
        // TODO: check that it is noninferred
        Ok(id)
    }
}
