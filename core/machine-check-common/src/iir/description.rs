use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::iir::{func::IFn, path::IIdent, ty::IElementaryType};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IDescription {
    pub structs: IndexMap<IIdent, IStruct>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IStruct {
    pub fields: IndexMap<IIdent, IElementaryType>,
    pub impls: IndexMap<IImplTrait, IImpl>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IImplTrait {
    Inherent,
    Machine,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct IImpl {
    pub fns: IndexMap<IIdent, IFn>,
}
