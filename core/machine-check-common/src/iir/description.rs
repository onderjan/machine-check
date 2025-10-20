use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::iir::{path::IIdent, ty::IElementaryType};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IDescription {
    pub structs: IndexMap<IIdent, IStruct>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IStruct {
    pub fields: IndexMap<IIdent, IElementaryType>,
}
