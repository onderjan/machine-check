use indexmap::IndexMap;

use crate::wir::{WFnSignature, WIdent, WPath, WTypeId, WUniquePath};

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
pub enum WSignature {
    Struct(WStructSig),
    Fn(WFnSig),
    Type(WTypeSig),
}

#[derive(Debug)]
pub struct WSignatures {
    inner: IndexMap<WUniquePath, WSignature>,
}

impl WSignatures {
    pub fn new() -> Self {
        Self {
            inner: IndexMap::new(),
        }
    }

    pub fn add_struct(&mut self, path: WUniquePath, signature: WStructSig) {
        self.inner.insert(path, WSignature::Struct(signature));
    }

    pub fn add_fn(&mut self, path: WUniquePath, signature: WFnSig) {
        self.inner.insert(path, WSignature::Fn(signature));
    }

    pub fn add_type(&mut self, path: WUniquePath, signature: WTypeSig) {
        self.inner.insert(path, WSignature::Type(signature));
    }

    pub fn get(&self, path: &WUniquePath) -> Option<&WSignature> {
        self.inner.get(path)
    }

    pub fn get_index(&self, index: usize) -> Option<(&WUniquePath, &WSignature)> {
        self.inner.get_index(index)
    }

    pub fn get_index_of(&self, path: &WUniquePath) -> Option<usize> {
        self.inner.get_index_of(path)
    }
}
