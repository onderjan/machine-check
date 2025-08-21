use crate::{
    iir::path::IPath,
    ir_common::{IrReference, IrTypeArray},
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum IElementaryType {
    Bitvector(u32),
    Array(IrTypeArray),
    Boolean,
    Path(IPath),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct IType {
    pub reference: IrReference,
    pub inner: IElementaryType,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum IGeneralType {
    Normal(IType),
    PanicResult(IType),
    PhiArg(IType),
}
