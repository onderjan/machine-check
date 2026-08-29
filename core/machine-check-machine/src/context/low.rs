use std::fmt::Debug;

use machine_check_common::{
    iir::ty::{IElementaryType, IGeneralType, IType},
    ir_common::IrReference,
};
use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, spanned::Spanned, AngleBracketedGenericArguments, Expr, ExprLit,
    GenericArgument, Ident, Item, Lit, LitInt, Path, PathArguments, PathSegment, Token, Type,
    TypePath, TypeReference,
};

use crate::wir::{WDatatype, WDatatypeId, WDefinitions, WTypeId, YSsa};

mod into_iir;

#[derive(Debug, Clone)]
pub struct WLowContext {
    definitions: WDefinitions<YSsa>,
    types: Vec<IGeneralType>,
}

impl WLowContext {
    pub(super) fn new(definitions: WDefinitions<YSsa>, types: Vec<IGeneralType>) -> Self {
        Self { definitions, types }
    }

    pub fn definitions(&self) -> &WDefinitions<YSsa> {
        &self.definitions
    }

    pub fn id_datatype(&self, id: WTypeId) -> &WDatatype {
        let ty = self.id_type(id);
        assert!(matches!(ty.reference, IrReference::None));
        let IElementaryType::Struct(struct_id) = ty.inner else {
            panic!("Definition type should be a struct");
        };

        let def_id = WDatatypeId::from_index(struct_id.0);

        let (_name, datatype) = self
            .definitions
            .datatype_by_id(def_id)
            .expect("Datatype should be found by id");

        datatype
    }

    pub fn id_general_type(&self, id: WTypeId) -> IGeneralType {
        self.types
            .get(id.0)
            .expect("Type id should be present")
            .clone()
    }

    pub fn id_type(&self, id: WTypeId) -> IType {
        let result = self.id_general_type(id);
        match result {
            IGeneralType::Normal(ty) => ty,
            _ => panic!("Expected normal IIR type, got {:?}", result),
        }
    }

    pub fn id_syn_type(&self, id: WTypeId) -> Type {
        let (is_phi_arg, itype) = match self.id_general_type(id) {
            IGeneralType::Normal(itype) => (false, itype),
            IGeneralType::PhiArg(itype) => (true, itype),
        };

        let span = Span::call_site();

        let result = match itype.inner {
            IElementaryType::Bitvector(width) => {
                let path = Path {
                    leading_colon: Some(Token![::](span)),
                    segments: Punctuated::from_iter([
                        PathSegment {
                            ident: Ident::new("mck", span),
                            arguments: PathArguments::None,
                        },
                        PathSegment {
                            ident: Ident::new("forward", span),
                            arguments: PathArguments::None,
                        },
                        PathSegment {
                            ident: Ident::new("Bitvector", span),
                            arguments: PathArguments::AngleBracketed(
                                AngleBracketedGenericArguments {
                                    colon2_token: None,
                                    lt_token: Token![<](span),
                                    args: Punctuated::from_iter([GenericArgument::Const(
                                        Expr::Lit(ExprLit {
                                            attrs: Vec::new(),
                                            lit: Lit::Int(LitInt::new(&width.to_string(), span)),
                                        }),
                                    )]),
                                    gt_token: Token![>](span),
                                },
                            ),
                        },
                    ]),
                };
                Type::Path(TypePath { qself: None, path })
            }

            IElementaryType::Array(_ir_type_array) => todo!("Array syn type"),
            IElementaryType::Boolean => {
                let path = Path {
                    leading_colon: Some(Token![::](span)),
                    segments: Punctuated::from_iter([
                        PathSegment {
                            ident: Ident::new("mck", span),
                            arguments: PathArguments::None,
                        },
                        PathSegment {
                            ident: Ident::new("forward", span),
                            arguments: PathArguments::None,
                        },
                        PathSegment {
                            ident: Ident::new("Boolean", span),
                            arguments: PathArguments::None,
                        },
                    ]),
                };
                Type::Path(TypePath { qself: None, path })
            }
            IElementaryType::Struct(struct_id) => {
                let Some((path, _datatype)) = self
                    .definitions
                    .datatype_by_id(WDatatypeId::from_index(struct_id.0))
                else {
                    todo!("Not a struct");
                };
                let path: Path = path.clone().into_path().into();
                Type::Path(TypePath { qself: None, path })
            }
        };
        let span = result.span();
        let result = match itype.reference {
            IrReference::Immutable => Type::Reference(TypeReference {
                and_token: Token![&](span),
                lifetime: None,
                mutability: None,
                elem: Box::new(result),
            }),
            IrReference::None => result,
        };

        if is_phi_arg {
            let phi_arg_path = Path {
                leading_colon: Some(Token![::](span)),
                segments: Punctuated::from_iter([
                    PathSegment {
                        ident: Ident::new("mck", span),
                        arguments: PathArguments::None,
                    },
                    PathSegment {
                        ident: Ident::new("forward", span),
                        arguments: PathArguments::None,
                    },
                    PathSegment {
                        ident: Ident::new("PhiArg", span),
                        arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: Token![<](span),
                            args: Punctuated::from_iter([GenericArgument::Type(result)]),
                            gt_token: Token![>](span),
                        }),
                    },
                ]),
            };
            Type::Path(TypePath {
                qself: None,
                path: phi_arg_path,
            })
        } else {
            result
        }
    }

    pub fn id_elementary_type(&self, id: WTypeId) -> IElementaryType {
        let result = self.id_type(id);
        if !matches!(result.reference, IrReference::None) {
            panic!(
                "Expected elementary type but received reference {:?}",
                result
            );
        }
        result.inner
    }

    pub fn into_syn(self) -> Vec<Item> {
        self.definitions
            .clone()
            .into_syn(&|type_id: WTypeId| -> Type {
                eprintln!("Into syn type id {:?}", type_id);
                let ty = self.id_syn_type(type_id);
                eprintln!("Type: {:?}", ty);
                ty
            })
    }
}
