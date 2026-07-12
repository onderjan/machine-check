use std::fmt::Debug;

use crate::wir::{
    context::{typedef::WTypeDefs, types::phi_arg_type_path},
    WSpan, WType, WTypeId,
};

mod partial;
mod typedef;
mod types;

pub use partial::WPartialContext;
pub use types::*;

#[derive(Debug)]
pub struct WContext {
    type_defs: WTypeDefs,
    types: Vec<WType>,
}

impl WContext {
    fn type_id(&mut self, ty: WType) -> WTypeId {
        let type_id = WTypeId(self.types.len());
        self.types.push(ty);
        type_id
    }

    pub fn phi_arg_id(&mut self, span: WSpan, inner: WTypeId) -> WTypeId {
        let inner = self.types[inner.0].clone();
        //"::mck::forward::PhiArg::phi"
        let ty = WType::Path(phi_arg_type_path(span, Some(inner)));
        self.type_id(ty)
    }
}
