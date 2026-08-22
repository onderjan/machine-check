use indexmap::IndexMap;
use proc_macro2::Span;
use syn::{Item, ItemMod, ItemUse, Token, Type, UseGlob};

use crate::wir::{IntoSyn, WImplItemType, WItemFn, WItemStruct, WTypeId, WUniquePath, YStage};

#[derive(Debug, Clone)]
pub enum WDefinition<Y: YStage> {
    Struct(WItemStruct),
    Fn(WItemFn<Y>),
    Type(WImplItemType),
}

impl<Y: YStage> WDefinition<Y> {
    pub fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Item {
        match self {
            WDefinition::Struct(item_struct) => Item::Struct(item_struct.into_syn(type_fn)),
            WDefinition::Fn(item_fn) => Item::Fn(item_fn.into_syn(type_fn)),
            WDefinition::Type(item_type) => {
                // TODO: correct
                Item::Use(ItemUse {
                    attrs: vec![],
                    vis: syn::Visibility::Inherited,
                    use_token: Token![use](Span::call_site()),
                    leading_colon: None,
                    tree: syn::UseTree::Glob(UseGlob {
                        star_token: Token![*](Span::call_site()),
                    }),
                    semi_token: Token![;](Span::call_site()),
                })
            } //Item::Type(item_type.into_syn(type_fn)),
        }
    }
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

    pub fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Vec<Item> {
        let mut items = Vec::new();
        for (_path, def) in self.inner {
            items.push(def.into_syn(type_fn));
        }
        items
    }
}
