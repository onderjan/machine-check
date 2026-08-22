use std::fmt::Debug;

use syn::Type;

use crate::{
    context::WInferenceContext,
    into_wir::{fold_type, Error},
    wir::{
        WDefinitions, WItemImpl, WItemStruct, WPartialType, WSpan, WTypeId, WUniquePath, YBuild,
        YTac,
    },
};

#[derive(Debug)]
pub struct WContextBuilder {
    definitions: WDefinitions<YBuild>,
    types: Vec<WPartialType>,
}

impl WContextBuilder {
    pub fn new() -> Self {
        Self {
            definitions: WDefinitions::new(),
            types: Vec::new(),
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

    pub fn noninferred_id(&mut self, ty: &Type) -> Result<WTypeId, Error> {
        let span = WSpan::from_syn(&ty);
        let ty = fold_type(ty.clone())?;
        if !ty.is_fully_inferred() {
            return Err(Error::new(
                crate::into_wir::ErrorType::IllegalConstruct(String::from(
                    "Interference not allowed here",
                )),
                span,
            ));
        }
        Ok(self.partial_type_id(ty))
    }

    fn partial_type_id(&mut self, ty: WPartialType) -> WTypeId {
        let id = WTypeId(self.types.len());
        self.types.push(ty);
        id
    }

    pub fn build(self) -> WInferenceContext {
        todo!("Build context")
    }
}
