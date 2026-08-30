use std::fmt::Debug;

mod attribute;
mod item;
mod item_fn;
mod path;

mod build;
mod property;

use syn::{Item, Type};

use crate::{
    wir::{
        WDefinitions, WFnId, WIdent, WInferenceType, WItemFn, WItemImpl, WItemStruct, WSpan,
        WStrippedPath, WType, WTypeId, WTypePath, WTypePathSegment, YBuild,
    },
    Error, ErrorType, Errors,
};

#[derive(Debug)]
pub struct WOuterContext {
    definitions: WDefinitions<YBuild>,
    types: Vec<WInferenceType>,
}

impl WOuterContext {
    pub fn new() -> Self {
        Self {
            definitions: WDefinitions::empty(),
            types: Vec::new(),
        }
    }

    pub fn add_fn(&mut self, item_fn: WItemFn<YBuild>) -> WFnId {
        let fn_name = item_fn.signature.ident.clone();
        self.definitions
            .add_fn(fn_name.into_path().without_generics(), item_fn)
    }

    fn known_type_id(&mut self, ty: WType) -> WTypeId {
        self.partial_type_id(WInferenceType::Inferred(ty))
    }

    fn partial_type_id(&mut self, ty: WInferenceType) -> WTypeId {
        let id = WTypeId::from_index(self.types.len());
        self.types.push(ty);
        id
    }

    fn total_syn_type_id(&mut self, ty: Type) -> Result<WTypeId, Error> {
        // TODO: ensure total
        let ty = self.fold_partial_type(ty)?;
        Ok(self.partial_type_id(ty))
    }

    fn partial_syn_type_id(&mut self, ty: Type) -> Result<WTypeId, Error> {
        let ty = self.fold_partial_type(ty)?;
        Ok(self.partial_type_id(ty))
    }

    pub fn new_bitvector(&mut self, width: Option<u32>) -> WTypeId {
        self.new_bitvector_like("Bitvector", width)
    }

    fn new_bitvector_like(&mut self, name: &str, width: Option<u32>) -> WTypeId {
        //let arg = WPartialPathArgument::Uint(width, WSpan::call_site());
        let generics = if let Some(width) = width {
            vec![self.partial_type_id(WInferenceType::Inferred(WType::Number(
                width,
                WSpan::call_site(),
            )))]
        } else {
            vec![]
        };
        let ty = WInferenceType::Inferred(WType::Path(WTypePath {
            leading_colon: Some(WSpan::call_site()),
            segments: vec![
                WTypePathSegment {
                    ident: WIdent::new(String::from("machine_check"), WSpan::call_site()),
                    generics: None,
                },
                WTypePathSegment {
                    ident: WIdent::new(String::from(name), WSpan::call_site()),
                    generics: Some(generics),
                },
            ],
        }));

        self.partial_type_id(ty)
    }

    pub fn new_bool(&mut self) -> WTypeId {
        let ty = WInferenceType::Inferred(WType::Path(WTypePath {
            leading_colon: None,
            segments: vec![WTypePathSegment {
                ident: WIdent::new(String::from("bool"), WSpan::call_site()),
                generics: None,
            }],
        }));
        self.partial_type_id(ty)
    }

    fn wildcard_id(&mut self, span: WSpan) -> WTypeId {
        self.partial_type_id(WInferenceType::Infer(span))
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
                let path = WIdent::from_syn_ident(item.ident.clone()).into_stripped_path();
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
                ErrorType::IllegalConstruct(String::from("Unknown self type")),
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
