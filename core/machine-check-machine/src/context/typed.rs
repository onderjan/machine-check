use std::fmt::Debug;

use crate::wir::{WDefinitions, WIdent, WType, WTypeId, WTypePath, WTypePathSegment, YTac};

mod convert;
mod lower;

#[derive(Debug)]
pub struct WTypedContext {
    definitions: WDefinitions<YTac>,
    types: Vec<crate::wir::WType>,
    boolean_type_id: WTypeId,
    panic_type_id: WTypeId,
}

impl WTypedContext {
    pub(super) fn new(
        definitions: WDefinitions<YTac>,
        types: Vec<WType>,
        boolean_type_id: WTypeId,
        panic_type_id: WTypeId,
    ) -> Self {
        Self {
            definitions,
            types,
            boolean_type_id,
            panic_type_id,
        }
    }

    pub fn boolean_type_id(&self) -> WTypeId {
        self.boolean_type_id.clone()
    }

    pub fn panic_type_id(&self) -> WTypeId {
        self.panic_type_id.clone()
    }

    pub fn wir_type(&self, id: WTypeId) -> WType {
        self.types[id.index()].clone()
    }

    pub fn definitions(&self) -> &WDefinitions<YTac> {
        &self.definitions
    }

    fn new_type_id(&mut self, ty: WType) -> WTypeId {
        let type_id = WTypeId::from_index(self.types.len());
        self.types.push(ty);
        type_id
    }

    fn new_phi_arg_id(&mut self, inner: WTypeId) -> WTypeId {
        let inner_ty = self.types[inner.index()].clone();
        let span = inner_ty.span();

        let ty = WType::Path(WTypePath {
            leading_colon: Some(span),
            segments: vec![
                WTypePathSegment {
                    ident: WIdent::new(String::from("mck"), span),
                    generics: None,
                },
                WTypePathSegment {
                    ident: WIdent::new(String::from("forward"), span),
                    generics: None,
                },
                WTypePathSegment {
                    ident: WIdent::new(String::from("PhiArg"), span),
                    generics: Some(vec![inner]),
                },
            ],
        });
        self.new_type_id(ty)
    }

    fn new_panic_result_id(&mut self, inner: WTypeId) -> WTypeId {
        let inner_ty = self.types[inner.index()].clone();
        let span = inner_ty.span();

        let ty = WType::Path(WTypePath {
            leading_colon: Some(span),
            segments: vec![
                WTypePathSegment {
                    ident: WIdent::new(String::from("mck"), span),
                    generics: None,
                },
                WTypePathSegment {
                    ident: WIdent::new(String::from("concr"), span),
                    generics: None,
                },
                WTypePathSegment {
                    ident: WIdent::new(String::from("PanicResult"), span),
                    generics: Some(vec![inner]),
                },
            ],
        });
        self.new_type_id(ty)
    }
}
