use std::fmt::Debug;

use machine_check_common::{
    iir::ty::{IElementaryType, IGeneralType, IType},
    ir_common::IrReference,
};
use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, spanned::Spanned, AngleBracketedGenericArguments, Expr, ExprLit,
    GenericArgument, Ident, Lit, LitInt, Path, PathArguments, PathSegment, Token, Type, TypePath,
    TypeReference,
};

use crate::wir::{WDefinition, WDefinitions, WTypeId};

#[derive(Debug)]
pub struct WLowContext {
    signatures: WDefinitions,
    types: Vec<IGeneralType>,
}

impl WLowContext {
    pub(super) fn new(signatures: WDefinitions, types: Vec<IGeneralType>) -> Self {
        Self { signatures, types }
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
                let Some((path, WDefinition::Struct(_struct_sig))) =
                    self.signatures.get_index(struct_id.0)
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

    fn new_type_id(&mut self, ty: IGeneralType) -> WTypeId {
        let type_id = WTypeId(self.types.len());
        self.types.push(ty);
        type_id
    }

    pub fn new_phi_arg_id(&mut self, inner: WTypeId) -> WTypeId {
        let IGeneralType::Normal(inner) = self.types[inner.0].clone() else {
            panic!("Expected phi inner to be normal");
        };

        let ty = IGeneralType::PhiArg(inner);
        self.new_type_id(ty)
    }

    pub fn new_bool_id(&mut self) -> WTypeId {
        let ty = IGeneralType::Normal(IType {
            reference: IrReference::None,
            inner: IElementaryType::Boolean,
        });
        self.new_type_id(ty)
    }

    /*
    fn type_id(&mut self, ty: WType) -> WTypeId {
        let type_id = WTypeId(self.types.len());
        self.types.push(ty);
        type_id
    }

    pub fn phi_arg_id(&mut self, span: WSpan, inner: WTypeId) -> WTypeId {
        let inner = self.types[inner.0].clone();
        //"::mck::forward::PhiArg::phi"
        let ty = WType::Path(phi_arg_type_path(span, Some(inner)));
        self.type_id(ty)
    }

    pub fn iir_id_general_type(&self, id: WTypeId) -> IGeneralType {
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
