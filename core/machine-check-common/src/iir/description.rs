use std::fmt::Debug;

use mck::refin::{RArray, RBitvector, RefinementValue};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::iir::{
    func::{IFn, IFnDeclaration},
    path::IIdent,
    ty::IElementaryType,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IMachine {
    pub machine: IStructId,
    pub init: IFnId,
    pub next: IFnId,

    pub input: IStructId,
    pub param: IStructId,
    pub state: IStructId,

    pub description: IDescription,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IDescription {
    pub structs: IndexMap<IIdent, IStruct>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IStructId(pub usize);

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

impl IMachine {
    pub fn machine(&self) -> &IStruct {
        self.description.struct_with_id(self.machine)
    }

    pub fn input(&self) -> &IStruct {
        self.description.struct_with_id(self.input)
    }

    pub fn param(&self) -> &IStruct {
        self.description.struct_with_id(self.param)
    }

    pub fn state(&self) -> &IStruct {
        self.description.struct_with_id(self.state)
    }

    pub fn init(&self) -> &IFn {
        self.description.fn_with_id(self.init)
    }
    pub fn next(&self) -> &IFn {
        self.description.fn_with_id(self.next)
    }
}

impl IDescription {
    pub fn struct_with_id(&self, id: IStructId) -> &IStruct {
        self.structs
            .get_index(id.0)
            .expect("Struct with given id should exist")
            .1
    }

    pub fn fn_with_id(&self, id: IFnId) -> &IFn {
        self.struct_with_id(id.struct_id)
            .fns
            .get_index(id.fn_index)
            .expect("Call with given id should exist")
            .1
    }
}

impl IStruct {
    pub fn clean_refin(&self) -> RefinementValue {
        let mut result = Vec::new();
        for field_ty in self.fields.values() {
            let field_result = match field_ty {
                IElementaryType::Bitvector(width) => {
                    RefinementValue::Bitvector(RBitvector::new_unmarked(*width))
                }
                IElementaryType::Array(array) => RefinementValue::Array(RArray::new_unmarked(
                    array.index_width,
                    array.element_width,
                )),
                IElementaryType::Boolean => todo!(),
                IElementaryType::Path(_) => todo!(),
            };
            result.push(field_result)
        }

        RefinementValue::Struct(result)
    }

    pub fn dirty_refin(&self) -> RefinementValue {
        let mut result = Vec::new();
        for field_ty in self.fields.values() {
            let field_result = match field_ty {
                IElementaryType::Bitvector(width) => {
                    RefinementValue::Bitvector(RBitvector::new_marked_unimportant(*width))
                }
                IElementaryType::Array(array) => RefinementValue::Array(
                    RArray::new_marked_unimportant(array.index_width, array.element_width),
                ),
                IElementaryType::Boolean => todo!(),
                IElementaryType::Path(_) => todo!(),
            };
            result.push(field_result)
        }

        RefinementValue::Struct(result)
    }
}

impl Debug for IStructId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IFnId {
    pub struct_id: IStructId,
    pub fn_index: usize,
}

impl Debug for IFnId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}::{}", self.struct_id, self.fn_index)
    }
}
