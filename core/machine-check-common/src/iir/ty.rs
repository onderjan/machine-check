use std::fmt::Debug;

use crate::{
    iir::path::IPath,
    ir_common::{IrReference, IrTypeArray},
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum IElementaryType {
    Bitvector(u32),
    Array(IrTypeArray),
    Boolean,
    Path(IPath),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct IType {
    pub reference: IrReference,
    pub inner: IElementaryType,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum IGeneralType {
    Normal(IType),
    PanicResult(IType),
    PhiArg(IType),
}

impl Debug for IElementaryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bitvector(width) => write!(f, "::mck::Bitvector<{}>", width),
            Self::Array(array) => write!(
                f,
                "::mck::Array<{}, {}>",
                array.index_width, array.element_width
            ),
            Self::Boolean => write!(f, "Boolean"),
            Self::Path(path) => path.fmt(f),
        }
    }
}

impl Debug for IType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.reference {
            IrReference::Immutable => f.write_str("&")?,
            IrReference::None => {}
        }
        self.inner.fmt(f)
    }
}

impl Debug for IGeneralType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal(inner) => write!(f, "{:?}", inner),
            Self::PanicResult(inner) => write!(f, "::mck::PanicResult<{:?}>", inner),
            Self::PhiArg(inner) => write!(f, "::mck::PanicResult<{:?}>", inner),
        }
    }
}
