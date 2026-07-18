use std::fmt::Debug;

use crate::{
    into_wir::Error,
    wir::{
        context::{typedef::WTypeDefs, types::phi_arg_type_path},
        WPathArgument, WSpan, WType, WTypeId,
    },
};

mod convert;
mod partial;
mod typedef;
mod types;

use machine_check_common::iir::ty::IElementaryType;
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

    pub fn iir_type(&self, id: WTypeId) -> IElementaryType {
        iir_ty(self.types.get(id.0).expect("Type id should be present"))
    }
}

fn iir_ty(ty: &WType) -> IElementaryType {
    match ty {
        WType::Path(path) => {
            let mut result = None;
            if path.matches_absolute(&["mck", "forward", "Bitvector"]) {
                if let Some(generics) = &path.segments[2].generics {
                    if generics.arguments.len() == 1 {
                        if let WPathArgument::Uint(width, _span) = generics.arguments[0] {
                            result = Some(IElementaryType::Bitvector(width))
                        }
                    }
                }
            }
            if let Some(result) = result {
                result
            } else {
                panic!("Cannot convert type to IIR: {:?}", path)
            }
        }
        WType::Reference(inner) => {
            todo!("IIR from reference to {:?}", inner)
        }
    }
}
