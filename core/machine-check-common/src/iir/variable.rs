use crate::iir::{path::IIdent, ty::IGeneralType};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IVarId(pub usize);

#[derive(Clone, Debug, Hash)]
pub struct IVarInfo {
    pub ident: IIdent,
    pub ty: IGeneralType,
}
