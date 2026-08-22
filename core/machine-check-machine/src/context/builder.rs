use std::fmt::Debug;

use indexmap::IndexMap;
use syn::Type;

mod expr;
mod func;
mod stmt;

use crate::{
    context::WInferenceContext,
    into_wir::{fold_type, Error, Errors},
    util::ident_creator::IdentCreator,
    wir::{
        WDefinition, WDefinitions, WIdent, WItemFn, WItemImpl, WItemStruct, WPartialType, WPath,
        WSpan, WTypeId, WUniquePath, YBuild, YTac,
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
            definitions: WDefinitions::empty(),
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

    pub fn type_id(&mut self, ty: &Type) -> Result<WTypeId, Error> {
        let ty = fold_type(ty.clone())?;
        Ok(self.partial_type_id(ty))
    }

    pub fn build(mut self) -> Result<WInferenceContext, Errors> {
        let mut definitions = Vec::new();
        for (path, def) in self.definitions.clone().into_inner() {
            let def = match def {
                WDefinition::Struct(item_struct) => Ok(WDefinition::Struct(item_struct)),
                WDefinition::Fn(item_fn) => self.build_function(item_fn).map(WDefinition::Fn),
                WDefinition::Type(item_type) => Ok(WDefinition::Type(item_type)),
            };
            definitions.push(def.map(|def| (path, def)));
        }
        let definitions = Errors::flat_result(definitions)?;
        let definitions = IndexMap::from_iter(definitions);

        Ok(WInferenceContext::new(
            WDefinitions::new(definitions),
            self.types,
        ))
    }

    fn build_function(&mut self, item_fn: WItemFn<YBuild>) -> Result<WItemFn<YTac>, Errors> {
        FunctionFolder {
            ctx: self,
            self_ty: None,
            ident_creator: IdentCreator::new(String::from("")),
            scopes: Vec::new(),
            local_types: IndexMap::new(),
            next_scope_id: 0,
        }
        .fold(item_fn)
    }
    fn wildcard_id(&mut self, span: WSpan) -> WTypeId {
        self.partial_type_id(WPartialType::Infer(span))
    }
}

struct FunctionScope {
    local_map: IndexMap<WIdent, WIdent>,
}

struct FunctionFolder<'a> {
    ctx: &'a mut WContextBuilder,
    self_ty: Option<(&'a Type, &'a WPath)>,
    ident_creator: IdentCreator<()>,
    local_types: IndexMap<WIdent, WTypeId>,
    scopes: Vec<FunctionScope>,
    next_scope_id: u32,
}
