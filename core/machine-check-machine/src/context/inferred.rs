use std::fmt::Debug;

use proc_macro2::Span;

use crate::{
    context::{
        inferred::{convert::convert_item_fn, lower::lower_item_fn},
        WLowContext,
    },
    into_wir::Errors,
    wir::{
        WDefinitions, WIdent, WItemFn, WPath, WPathArgument, WPathGenerics, WPathSegment, WType,
        WTypeId, YSsa, YTac,
    },
};

mod convert;
mod lower;

#[derive(Debug)]
pub struct WInferredContext {
    definitions: WDefinitions<YTac>,
    types: Vec<WType>,
    boolean_type_id: WTypeId,
    panic_type_id: WTypeId,
}

impl WInferredContext {
    pub(super) fn new(
        definitions: WDefinitions<YTac>,
        types: Vec<WType>,
        boolean_type_id: WTypeId,
        panic_type_id: WTypeId,
    ) -> Self {
        eprintln!("Num inferred types: {}", types.len());
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

        /*for (path, def) in self.definitions.clone().into_inner() {
            let def = match def {
                WDefinition::Struct(item_struct) => WDefinition::Struct(item_struct),
                WDefinition::Fn(item_fn) => {
                    let item_fn = lower_item_fn(&mut self, item_fn)?;
                    let item_fn = convert_item_fn(&mut self, item_fn)?;
                    WDefinition::Fn(item_fn)
                }
                WDefinition::Type(impl_item_type) => WDefinition::Type(impl_item_type),
            };
            definitions.insert(path, def);
        }*/

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

    fn new_type_id(&mut self, ty: WType) -> WTypeId {
        let type_id = WTypeId(self.types.len());
        self.types.push(ty);
        type_id
    }

    fn new_phi_arg_id(&mut self, inner: WTypeId) -> WTypeId {
        let inner = self.types[inner.0].clone();
        let span = inner.wir_span();

        let ty = WType::Path(WPath {
            leading_colon: Some(span),
            segments: vec![
                WPathSegment {
                    ident: WIdent::new(String::from("mck"), span.first()),
                    generics: None,
                },
                WPathSegment {
                    ident: WIdent::new(String::from("forward"), span.first()),
                    generics: None,
                },
                WPathSegment {
                    ident: WIdent::new(String::from("PhiArg"), span.first()),
                    generics: Some(WPathGenerics {
                        turbofish: Some(span),
                        arguments: vec![WPathArgument::Type(inner)],
                    }),
                },
            ],
        });
        self.new_type_id(ty)
    }

    fn new_bool_id(&mut self) -> WTypeId {
        let ty = WType::Path(WPath {
            leading_colon: None,
            segments: vec![WPathSegment {
                ident: WIdent::new(String::from("bool"), Span::call_site()),
                generics: None,
            }],
        });
        self.new_type_id(ty)
    }

    /*pub fn iir_id_general_type(&self, id: WTypeId) -> IGeneralType {
        self.iir_ty(self.types.get(id.0).expect("Type id should be present"))
    }

    pub fn iir_id_type(&self, id: WTypeId) -> IType {
        let result = self.iir_id_general_type(id);
        match result {
            IGeneralType::Normal(ty) => ty,
            _ => panic!("Expected normal IIR type, got {:?}", result),
        }
    }

    pub fn iir_id_elementary_type(&self, id: WTypeId) -> IElementaryType {
        let result = self.iir_id_type(id);
        if !matches!(result.reference, IrReference::None) {
            panic!(
                "Expected elementary type but received reference {:?}",
                result
            );
        }
        result.inner
    }

    pub fn register_iir_id(&mut self, ty: Type, id: IStructId) {
        self.iir_registrations.insert(ty, id);
    }

    fn iir_ty(&self, ty: &WType) -> IGeneralType {
        match ty {
            WType::Path(path) => {
                if path.matches_absolute(&["mck", "forward", "Bitvector"]) {
                    if let Some(generics) = &path.segments[2].generics {
                        if generics.arguments.len() == 1 {
                            if let WPathArgument::Uint(width, _span) = generics.arguments[0] {
                                return IGeneralType::Normal(IType {
                                    reference: IrReference::None,
                                    inner: IElementaryType::Bitvector(width),
                                });
                            }
                        }
                    }
                }

                if path.matches_absolute(&["mck", "forward", "PhiArg"]) {
                    if let Some(generics) = &path.segments[2].generics {
                        if generics.arguments.len() == 1 {
                            if let WPathArgument::Type(ty) = &generics.arguments[0] {
                                let inner = self.iir_ty(&ty);
                                let inner = match inner {
                                    IGeneralType::Normal(ty) => ty,
                                    _ => panic!(
                                        "Expected normal IIR as phi arg inner, got {:?}",
                                        inner
                                    ),
                                };
                                return IGeneralType::PhiArg(inner);
                            }
                        }
                    }
                }

                if path.matches_relative(&["bool"]) {
                    return IGeneralType::Normal(IType {
                        reference: IrReference::None,
                        inner: IElementaryType::Boolean,
                    });
                }

                let syn_path: Path = Path::from(path.clone());
                let ty = Type::Path(TypePath {
                    path: syn_path,
                    qself: None,
                });

                if let Some(iir_id) = self.iir_registrations.get(&ty) {
                    IGeneralType::Normal(IType {
                        reference: IrReference::None,
                        inner: IElementaryType::Struct(*iir_id),
                    })
                } else {
                    panic!("Cannot convert type to IIR: {:?}", path)
                }
            }
            WType::Reference(inner) => {
                let inner = self.iir_ty(inner.as_ref());
                let mut inner = match inner {
                    IGeneralType::Normal(ty) => ty,
                    _ => panic!("Expected normal IIR as reference inner, got {:?}", inner),
                };
                assert!(matches!(inner.reference, IrReference::None));
                inner.reference = IrReference::Immutable;
                IGeneralType::Normal(inner)
            }
        }
    }*/
}
