use std::fmt::Debug;

use crate::{
    context::{
        inferred::{convert::convert_item_fn, lower::lower_item_fn},
        WLowContext,
    },
    into_wir::Errors,
    wir::{
        WDefinitions, WIdent, WItemFn, WTotalPath, WTotalPathArgument, WTotalPathGenerics,
        WTotalPathSegment, WTotalType, WTypeId, YSsa, YTac,
    },
};

mod convert;
mod lower;

#[derive(Debug)]
pub struct WInferredContext {
    definitions: WDefinitions<YTac>,
    types: Vec<WTotalType>,
    boolean_type_id: WTypeId,
    panic_type_id: WTypeId,
}

impl WInferredContext {
    pub(super) fn new(
        definitions: WDefinitions<YTac>,
        types: Vec<WTotalType>,
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

    pub fn wir_type(&self, id: WTypeId) -> WTotalType {
        self.types[id.0].clone()
    }

    pub fn definitions(&self) -> &WDefinitions<YTac> {
        &self.definitions
    }

    pub fn lower(mut self) -> Result<WLowContext, Errors> {
        let definitions = self
            .definitions
            .clone()
            .map_functions(|func| self.lower_function(func))?;

        let mut types = Vec::new();

        for ty in &self.types {
            let lowered = self.lower_type(ty.clone())?;
            eprintln!("Lowered type to: {:?}", lowered);
            types.push(lowered);
        }

        Ok(WLowContext::new(definitions, types))
    }

    fn lower_function(&mut self, item_fn: WItemFn<YTac>) -> Result<WItemFn<YSsa>, Errors> {
        let item_fn = lower_item_fn(self, item_fn)?;
        convert_item_fn(self, item_fn)
    }

    fn new_type_id(&mut self, ty: WTotalType) -> WTypeId {
        let type_id = WTypeId(self.types.len());
        self.types.push(ty);
        type_id
    }

    fn new_phi_arg_id(&mut self, inner: WTypeId) -> WTypeId {
        let inner = self.types[inner.0].clone();
        let span = inner.wir_span();

        let ty = WTotalType::Path(WTotalPath {
            leading_colon: Some(span),
            segments: vec![
                WTotalPathSegment {
                    ident: WIdent::new(String::from("mck"), span.first()),
                    generics: None,
                },
                WTotalPathSegment {
                    ident: WIdent::new(String::from("forward"), span.first()),
                    generics: None,
                },
                WTotalPathSegment {
                    ident: WIdent::new(String::from("PhiArg"), span.first()),
                    generics: Some(WTotalPathGenerics {
                        turbofish: Some(span),
                        arguments: vec![WTotalPathArgument::Type(inner)],
                    }),
                },
            ],
        });
        self.new_type_id(ty)
    }
}
