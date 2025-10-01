use std::fmt::Debug;

use crate::iir::{path::IIdent, ty::IGeneralType};

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IVarId(pub usize);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IVarInfo {
    pub ident: IIdent,
    pub ty: IGeneralType,
}

impl Debug for IVarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl Debug for IVarInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.ident.fmt(f)?;
        f.write_str(": ")?;
        self.ty.fmt(f)
    }
}
