use proc_macro2::Span;
use syn::{Token, Type, TypeInfer};

use crate::wir::{WPath, WTypeId};

#[derive(Clone, Debug)]
pub enum WContextType {
    Unresolved(Box<Type>),
    Resolved,
}

#[derive(Debug)]
pub struct WContext {
    types: Vec<WContextType>,
}

pub struct RequiresInferenceError;

impl WContext {
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }

    pub fn get_type(&mut self, ty: &Type) -> WTypeId {
        let id = WTypeId(self.types.len());
        self.types
            .push(WContextType::Unresolved(Box::new(ty.clone())));
        id
    }

    pub fn get_noninferred_type(&mut self, ty: &Type) -> Result<WTypeId, RequiresInferenceError> {
        // TODO: check that it is noninferred
        Ok(self.get_type(ty))
    }

    pub fn get_noninferred_type_path(
        &mut self,
        path: &WPath,
    ) -> Result<WTypeId, RequiresInferenceError> {
        let id = WTypeId(self.types.len());
        // TODO: check that it is noninferred
        Ok(id)
    }

    pub fn infer_type(&mut self, span: Span) -> WTypeId {
        self.get_type(&Type::Infer(TypeInfer {
            underscore_token: Token![_](span),
        }))
    }
}
