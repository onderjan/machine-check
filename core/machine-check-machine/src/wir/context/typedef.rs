use std::fmt::Debug;

use indexmap::IndexMap;
use machine_check_common::iir::description::IStructId;
use quote::quote;
use syn::Type;

use crate::wir::{WIdent, WTypeId};

#[derive(Clone, Debug)]
pub enum WContextTypeDef {
    Struct(Vec<(WIdent, WTypeId)>),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WContextSynType(Type);

impl Debug for WContextSynType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = &self.0;
        write!(f, "{}", quote!(#ty))
    }
}

#[derive(Debug, Clone)]
pub struct WTypeDefs {
    inner: IndexMap<WContextSynType, WContextTypeDef>,
}

impl WTypeDefs {
    pub fn new() -> Self {
        Self {
            inner: IndexMap::new(),
        }
    }

    pub fn add(&mut self, ty: Type, def: WContextTypeDef) {
        self.inner.insert(WContextSynType(ty), def);
    }

    pub fn get(&self, ty: &Type) -> Option<&WContextTypeDef> {
        self.inner.get(&WContextSynType(ty.clone()))
    }

    pub fn get_index_of(&self, ty: &Type) -> Option<usize> {
        self.inner.get_index_of(&WContextSynType(ty.clone()))
    }

    pub fn into_inner(self) -> IndexMap<WContextSynType, WContextTypeDef> {
        self.inner
    }
}
