use indexmap::IndexMap;

use crate::wir::{WImplItemType, WItemFn, WItemStruct, WUniquePath, YStage};

#[derive(Debug, Clone)]
pub enum WDefinition<Y: YStage> {
    Struct(WItemStruct),
    Fn(WItemFn<Y>),
    Type(WImplItemType),
}

#[derive(Debug, Clone)]
pub struct WDefinitions<Y: YStage> {
    inner: IndexMap<WUniquePath, WDefinition<Y>>,
}

impl<Y: YStage> WDefinitions<Y> {
    pub fn new(inner: IndexMap<WUniquePath, WDefinition<Y>>) -> Self {
        Self { inner }
    }
    pub fn empty() -> Self {
        Self::new(IndexMap::new())
    }

    pub fn inner(&self) -> &IndexMap<WUniquePath, WDefinition<Y>> {
        &self.inner
    }

    pub fn into_inner(self) -> IndexMap<WUniquePath, WDefinition<Y>> {
        self.inner
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
