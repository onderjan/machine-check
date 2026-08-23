use std::fmt::Debug;

use indexmap::IndexMap;
use proc_macro2::Span;
use syn::{punctuated::Punctuated, Ident, Path, PathArguments, PathSegment, Type, TypePath};

mod expr;
mod func;
mod stmt;

use crate::{
    context::WInferenceContext,
    into_wir::{fold_type, Error, Errors},
    util::ident_creator::IdentCreator,
    wir::{
        WDefinitions, WFnId, WIdent, WItemFn, WItemImpl, WItemStruct, WPartialType, WPath,
        WPathSegment, WSpan, WSpanned, WTypeId, WUniquePath, YBuild, YTac,
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

    pub fn add_impl(&mut self, item_impl: WItemImpl<YBuild>) -> Result<(), Error> {
        let datatype_path = item_impl.self_ty.clone().without_generics();
        let Some(self_datatype) = self.definitions.datatype_id(&datatype_path) else {
            return Err(Error::new(
                crate::into_wir::ErrorType::IllegalConstruct(String::from("Unknown self type")),
                item_impl.wir_span(),
            ));
        };

        for impl_type in item_impl.impl_item_types {
            let assoc_name = impl_type.left_ident.clone();
            self.definitions.add_assoc_type(
                self_datatype,
                item_impl.trait_.clone(),
                assoc_name,
                impl_type,
            );
        }

        for impl_fn in item_impl.impl_item_fns {
            let fn_name = impl_fn.signature.ident.clone();
            self.definitions
                .add_impl_fn(self_datatype, item_impl.trait_.clone(), fn_name, impl_fn);
        }

        Ok(())
    }

    pub fn add_fn(&mut self, item_fn: WItemFn<YBuild>) -> WFnId {
        let fn_name = item_fn.signature.ident.clone();
        self.definitions.add_fn(fn_name.into_path(), item_fn)
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

    pub fn bool_type_id(&mut self) -> WTypeId {
        let ty = &Type::Path(TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: Punctuated::from_iter([PathSegment {
                    ident: Ident::new("bool", Span::call_site()),
                    arguments: PathArguments::None,
                }]),
            },
        });
        self.type_id(ty)
            .expect("Bool type should be assigned a type id")
    }

    pub fn build(mut self) -> Result<WInferenceContext, Errors> {
        let definitions = self
            .definitions
            .clone()
            .map_functions(|func| self.build_function(func))?;
        /*let mut definitions = Vec::new();

        for (path, def) in self.definitions.clone().into_inner() {
            let def = match def {
                WDefinition::Struct(item_struct) => Ok(WDefinition::Struct(item_struct)),
                WDefinition::Fn(item_fn) => self.build_function(item_fn).map(WDefinition::Fn),
                WDefinition::Type(item_type) => Ok(WDefinition::Type(item_type)),
            };
            definitions.push(def.map(|def| (path, def)));
        }
        let definitions = Errors::flat_result(definitions)?;
        let definitions = IndexMap::from_iter(definitions);*/

        Ok(WInferenceContext::new(definitions, self.types))
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
