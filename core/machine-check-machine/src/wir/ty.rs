use std::fmt::Debug;
use syn::Type;

use super::IntoTypedSyn;

mod partial;
mod total;

pub use partial::*;
pub use total::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WTypeId(usize);

impl WTypeId {
    pub fn from_index(index: usize) -> WTypeId {
        Self(index)
    }

    pub fn index(&self) -> usize {
        self.0
    }
}

impl IntoTypedSyn<Type> for WTypeId {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Type {
        type_fn(self)
    }
}

impl Debug for WTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}
