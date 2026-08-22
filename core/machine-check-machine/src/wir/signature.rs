use indexmap::IndexMap;

use crate::wir::{WIdent, WImplItemType, WItemFn, WItemStruct, WTypeId, WUniquePath, YStage};

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
pub enum WDefinition<Y: YStage> {
    Struct(WItemStruct),
    Fn(WItemFn<Y>),
    Type(WImplItemType),
}

#[derive(Debug)]
pub struct WDefinitions<Y: YStage> {
    inner: IndexMap<WUniquePath, WDefinition<Y>>,
}

impl<Y: YStage> WDefinitions<Y> {
    pub fn new() -> Self {
        Self {
            inner: IndexMap::new(),
        }
    }

    pub fn add_struct(&mut self, path: WUniquePath, def: WItemStruct) {
        self.inner.insert(path, WDefinition::Struct(def));
    }

    pub fn add_fn(&mut self, path: WUniquePath, def: WItemFn<Y>) {
        self.inner.insert(path, WDefinition::Fn(def));
    }

    pub fn add_type(&mut self, path: WUniquePath, def: WImplItemType) {
        self.inner.insert(path, WDefinition::Type(def));
    }

    pub fn get(&self, path: &WUniquePath) -> Option<&WDefinition<Y>> {
        self.inner.get(path)
    }

    pub fn get_index(&self, index: usize) -> Option<(&WUniquePath, &WDefinition<Y>)> {
        self.inner.get_index(index)
    }

    pub fn get_index_of(&self, path: &WUniquePath) -> Option<usize> {
        self.inner.get_index_of(path)
    }
}
