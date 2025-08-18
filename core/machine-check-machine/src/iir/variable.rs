use crate::wir::{WElementaryType, WGeneralType, WIdent};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IVarId(pub usize);

#[derive(Clone, Debug, Hash)]
pub struct IVarInfo {
    pub ident: WIdent,
    pub ty: WGeneralType<WElementaryType>,
}
