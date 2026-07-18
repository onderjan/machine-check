use std::fmt::Debug;

use crate::wir::{
    context::{typedef::WTypeDefs, types::phi_arg_type_path},
    WPathArgument, WSpan, WType, WTypeId,
};

mod convert;
mod inference;
mod typedef;
mod types;

use indexmap::IndexMap;
pub use inference::WInferenceContext;
use machine_check_common::{
    iir::{
        description::IStructId,
        ty::{IElementaryType, IGeneralType, IType},
    },
    ir_common::IrReference,
};
use syn::{Path, Type, TypePath};
pub use types::*;

#[derive(Debug)]
pub struct WContext {
    type_defs: WTypeDefs,
    types: Vec<WType>,
    iir_registrations: IndexMap<Type, IStructId>,
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

    pub fn iir_id_general_type(&self, id: WTypeId) -> IGeneralType {
        self.iir_ty(self.types.get(id.0).expect("Type id should be present"))
    }

    pub fn iir_id_type(&self, id: WTypeId) -> IType {
        let result = self.iir_id_general_type(id);
        match result {
            IGeneralType::Normal(ty) => ty,
            _ => panic!("Expected normal IIR type, got {:?}", result),
        }
    }

    pub fn iir_id_elementary_type(&self, id: WTypeId) -> IElementaryType {
        let result = self.iir_id_type(id);
        if !matches!(result.reference, IrReference::None) {
            panic!(
                "Expected elementary type but received reference {:?}",
                result
            );
        }
        result.inner
    }

    pub fn register_iir_id(&mut self, ty: Type, id: IStructId) {
        self.iir_registrations.insert(ty, id);
    }

    fn iir_ty(&self, ty: &WType) -> IGeneralType {
        match ty {
            WType::Path(path) => {
                if path.matches_absolute(&["mck", "forward", "Bitvector"]) {
                    if let Some(generics) = &path.segments[2].generics {
                        if generics.arguments.len() == 1 {
                            if let WPathArgument::Uint(width, _span) = generics.arguments[0] {
                                return IGeneralType::Normal(IType {
                                    reference: IrReference::None,
                                    inner: IElementaryType::Bitvector(width),
                                });
                            }
                        }
                    }
                }

                if path.matches_absolute(&["mck", "forward", "PhiArg"]) {
                    if let Some(generics) = &path.segments[2].generics {
                        if generics.arguments.len() == 1 {
                            if let WPathArgument::Type(ty) = &generics.arguments[0] {
                                let inner = self.iir_ty(&ty);
                                let inner = match inner {
                                    IGeneralType::Normal(ty) => ty,
                                    _ => panic!(
                                        "Expected normal IIR as phi arg inner, got {:?}",
                                        inner
                                    ),
                                };
                                return IGeneralType::PhiArg(inner);
                            }
                        }
                    }
                }

                if path.matches_relative(&["bool"]) {
                    return IGeneralType::Normal(IType {
                        reference: IrReference::None,
                        inner: IElementaryType::Boolean,
                    });
                }

                let syn_path: Path = Path::from(path.clone());
                let ty = Type::Path(TypePath {
                    path: syn_path,
                    qself: None,
                });

                if let Some(iir_id) = self.iir_registrations.get(&ty) {
                    IGeneralType::Normal(IType {
                        reference: IrReference::None,
                        inner: IElementaryType::Struct(*iir_id),
                    })
                } else {
                    panic!("Cannot convert type to IIR: {:?}", path)
                }
            }
            WType::Reference(inner) => {
                let inner = self.iir_ty(inner.as_ref());
                let mut inner = match inner {
                    IGeneralType::Normal(ty) => ty,
                    _ => panic!("Expected normal IIR as reference inner, got {:?}", inner),
                };
                assert!(matches!(inner.reference, IrReference::None));
                inner.reference = IrReference::Immutable;
                IGeneralType::Normal(inner)
            }
        }
    }
}
