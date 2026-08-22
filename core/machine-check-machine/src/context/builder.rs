use std::fmt::Debug;

use crate::wir::{WDefinitions, WItemImpl, WItemStruct, WUniquePath, YBuild, YTac};

#[derive(Debug)]
pub struct WContextBuilder {
    definitions: WDefinitions<YBuild>,
}

impl WContextBuilder {
    pub fn new() -> Self {
        Self {
            definitions: WDefinitions::new(),
        }
    }

    pub fn add_struct(&mut self, path: WUniquePath, item_struct: WItemStruct) {
        self.definitions.add_struct(path, item_struct);
    }

    pub fn add_impl(&mut self, path: WUniquePath, item_impl: WItemImpl<YBuild>) {
        for impl_type in item_impl.impl_item_types {
            let mut type_path = path.clone();
            type_path.segments.push(impl_type.left_ident.clone());
            self.definitions.add_type(type_path, impl_type);
        }

        for impl_fn in item_impl.impl_item_fns {
            let mut fn_path = path.clone();
            fn_path.segments.push(impl_fn.signature.ident.clone());
            self.definitions.add_fn(fn_path, impl_fn);
        }
    }
}
