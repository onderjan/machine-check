use std::fmt::Debug;
use syn::Type;

use super::IntoSyn;

mod partial;
mod total;

pub use partial::*;
pub use total::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WTypeId(pub usize);

impl IntoSyn<Type> for WTypeId {
    fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Type {
        type_fn(self)
    }
}

impl Debug for WTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}
