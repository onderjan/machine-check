use machine_check_common::iir::ty::{IElementaryType, IGeneralType, IType};

use crate::wir::{WElementaryType, WGeneralType, WType};

impl WElementaryType {
    pub fn into_iir(self) -> IElementaryType {
        match self {
            WElementaryType::Bitvector(width) => IElementaryType::Bitvector(width),
            WElementaryType::Array(type_array) => IElementaryType::Array(type_array),
            WElementaryType::Boolean => IElementaryType::Boolean,
            WElementaryType::Path(path) => IElementaryType::Path(path.into_iir()),
        }
    }
}

impl WType<WElementaryType> {
    pub fn into_iir(self) -> IType {
        IType {
            reference: self.reference,
            inner: self.inner.into_iir(),
        }
    }
}

impl WGeneralType<WElementaryType> {
    pub fn into_iir(self) -> IGeneralType {
        match self {
            WGeneralType::Normal(ty) => IGeneralType::Normal(ty.into_iir()),
            WGeneralType::PanicResult(ty) => IGeneralType::PanicResult(ty.into_iir()),
            WGeneralType::PhiArg(ty) => IGeneralType::PhiArg(ty.into_iir()),
        }
    }
}
