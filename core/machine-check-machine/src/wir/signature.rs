use indexmap::IndexMap;

use crate::wir::{WIdent, WImplItemType, WItemFn, WItemStruct, WTypeId, WUniquePath, YTac};

#[derive(Debug)]
pub struct WStructSig {
    pub fields: IndexMap<WIdent, WTypeId>,
}

#[derive(Debug)]
pub struct WTypeSig {
    pub inside_impl: bool,
}

#[derive(Debug)]
pub struct WFnSig {
    pub inputs: Vec<WTypeId>,
    pub output: WTypeId,
    pub inside_impl: bool,
}

#[derive(Debug)]
pub enum WDefinition {
    Struct(WItemStruct),
    Fn(WItemFn<YTac>),
    Type(WImplItemType),
}

#[derive(Debug)]
pub struct WDefinitions {
    inner: IndexMap<WUniquePath, WDefinition>,
}

impl WDefinitions {
    pub fn new() -> Self {
        Self {
            inner: IndexMap::new(),
        }
    }

    pub fn add_struct(&mut self, path: WUniquePath, def: WItemStruct) {
        self.inner.insert(path, WDefinition::Struct(def));
    }

    pub fn add_fn(&mut self, path: WUniquePath, def: WItemFn<YTac>) {
        self.inner.insert(path, WDefinition::Fn(def));
    }

    pub fn add_type(&mut self, path: WUniquePath, def: WImplItemType) {
        self.inner.insert(path, WDefinition::Type(def));
    }

    pub fn get(&self, path: &WUniquePath) -> Option<&WDefinition> {
        self.inner.get(path)
    }

    pub fn get_index(&self, index: usize) -> Option<(&WUniquePath, &WDefinition)> {
        self.inner.get_index(index)
    }

    pub fn get_index_of(&self, path: &WUniquePath) -> Option<usize> {
        self.inner.get_index_of(path)
    }
}
