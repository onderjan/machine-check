use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::iir::{
    func::{IFn, IFnDeclaration},
    path::IIdent,
    ty::IElementaryType,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IDescription {
    pub structs: IndexMap<IIdent, IStruct>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IStructDeclaration {
    pub fields: IndexMap<IIdent, IElementaryType>,
    pub fns: IndexMap<(ITrait, IIdent), IFnDeclaration>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IStruct {
    pub fields: IndexMap<IIdent, IElementaryType>,
    pub fns: IndexMap<(ITrait, IIdent), IFn>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ITrait {
    Inherent,
    Machine,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct IImpl {
    fns: IndexMap<IIdent, IFn>,
}
