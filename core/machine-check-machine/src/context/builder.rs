use std::fmt::Debug;

mod attribute;
mod item;
mod item_fn;
mod path;

mod build;

use syn::{Item, Type};

use crate::{
    into_wir::{Error, Errors},
    wir::{
        WDefinitions, WFnId, WIdent, WItemFn, WItemImpl, WItemStruct, WPartialType, WSpan,
        WStrippedPath, WTotalPath, WTotalPathSegment, WTotalType, WTypeId, YBuild,
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

    pub fn add_fn(&mut self, item_fn: WItemFn<YBuild>) -> WFnId {
        let fn_name = item_fn.signature.ident.clone();
        self.definitions.add_fn(fn_name.into_path(), item_fn)
    }

    pub fn total_syn_type_id(&mut self, ty: Type) -> Result<WTypeId, Error> {
        let ty = Self::fold_total_type(ty)?;
        Ok(self.partial_type_id(ty.into_partial()))
    }

    fn partial_syn_type_id(&mut self, ty: Type) -> Result<WTypeId, Error> {
        let ty = Self::fold_partial_type(ty)?;
        Ok(self.partial_type_id(ty))
    }

    pub fn partial_type_id(&mut self, ty: WPartialType) -> WTypeId {
        let id = WTypeId::from_index(self.types.len());
        self.types.push(ty);
        id
    }

    pub fn bool_type_id(&mut self) -> WTypeId {
        let ty = WTotalType::Path(WTotalPath {
            leading_colon: None,
            segments: vec![WTotalPathSegment {
                ident: WIdent::new(String::from("bool"), WSpan::call_site()),
                generics: None,
            }],
        });
        self.partial_type_id(ty.into_partial())
    }

    fn wildcard_id(&mut self, span: WSpan) -> WTypeId {
        self.partial_type_id(WPartialType::Infer(span))
    }

    pub fn add_syn_items(&mut self, items: Vec<Item>) -> Result<(), Errors> {
        let mut errors = Vec::new();

        for item in items {
            match self.add_syn_item(item) {
                Ok(()) => {}
                Err(errs) => errors.push(errs),
            }
        }

        Errors::errors_vec_to_result(errors)?;

        Ok(())
    }

    pub fn add_syn_item(&mut self, item: Item) -> Result<(), Errors> {
        let mut errors = Vec::new();

        match item {
            Item::Struct(item) => {
                let path = WTotalPath::from_ident(WIdent::from_syn_ident(item.ident.clone()))
                    .without_generics();
                match self.fold_item_struct(item) {
                    Ok(item_struct) => {
                        self.add_struct(path, item_struct);
                    }
                    Err(err) => errors.push(err),
                }
            }
            Item::Impl(item) => match self.fold_item_impl(item) {
                Ok(item_impl) => {
                    self.add_impl(item_impl)?;
                }
                Err(err) => errors.push(err),
            },
            _ => errors.push(Error::unsupported_syn_construct("Item kind", &item).into()),
        }

        Errors::errors_vec_to_result(errors)?;

        Ok(())
    }

    fn add_struct(&mut self, path: WStrippedPath, item_struct: WItemStruct) {
        self.definitions.add_struct(path, item_struct);
    }

    fn add_impl(&mut self, item_impl: WItemImpl<YBuild>) -> Result<(), Error> {
        let datatype_path = item_impl.self_ty.clone().without_generics();
        let Some(self_datatype) = self.definitions.datatype_id(&datatype_path) else {
            return Err(Error::new(
                crate::into_wir::ErrorType::IllegalConstruct(String::from("Unknown self type")),
                item_impl.span(),
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
}
