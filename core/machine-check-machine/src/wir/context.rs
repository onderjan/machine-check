use syn::Type;

use crate::wir::{WPath, WTypeDef, WTypeId};

pub struct WContext {
    types: Vec<WTypeDef>,
}

pub struct RequiresInferenceError;

impl WContext {
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }

    pub fn get_type(&mut self, ty: &Type) -> WTypeId {
        let id = WTypeId(self.types.len());
        id
    }

    pub fn get_noninferred_type(&mut self, ty: &Type) -> Result<WTypeId, RequiresInferenceError> {
        let id = WTypeId(self.types.len());
        // TODO: check that it is noninferred
        Ok(id)
    }

    pub fn get_noninferred_type_path(
        &mut self,
        path: &WPath,
    ) -> Result<WTypeId, RequiresInferenceError> {
        let id = WTypeId(self.types.len());
        // TODO: check that it is noninferred
        Ok(id)
    }

    pub fn wildcard_type(&mut self) -> WTypeId {
        let id = WTypeId(self.types.len());
        id
    }
}
