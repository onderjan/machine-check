use indexmap::IndexMap;

use crate::wir::{WPath, WTypeId};

#[derive(Debug)]
pub struct WStructSignature;

#[derive(Debug)]
pub struct WImplTypeSignature;

#[derive(Debug)]
pub struct WImplFnSignature {
    pub inputs: Vec<WTypeId>,
    pub output: WTypeId,
}

#[derive(Debug)]
pub enum WSignature {
    Struct(WStructSignature),
    ImplFn(WImplFnSignature),
    ImplType(WImplTypeSignature),
}

#[derive(Debug)]
pub struct WSignatures {
    inner: IndexMap<WPath, WSignature>,
}

impl WSignatures {
    pub fn new(inner: IndexMap<WPath, WSignature>) -> Self {
        Self { inner }
    }

    pub fn get(&self, path: &WPath) -> Option<&WSignature> {
        self.inner.get(path)
    }
}
