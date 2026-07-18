use std::fmt::Debug;

use indexmap::IndexMap;
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

#[derive(Debug)]
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
}
