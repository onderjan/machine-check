use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, Expr, ExprLit, GenericArgument, Ident,
    Lit, LitInt, Path, PathArguments, PathSegment, Token, Type, TypeInfer, TypePath, TypeReference,
};

use super::{IntoSyn, WPath};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WBasicType {
    Bitvector(u32),
    BitvectorArray(WTypeArray),
    Unsigned(u32),
    Signed(u32),
    Boolean,
    Path(WPath),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WGeneralType {
    Normal(WType),
    PanicResult(WType),
    PhiArg(WType),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WType {
    pub reference: WReference,
    pub inner: WBasicType,
}

impl WBasicType {
    pub fn into_type(self) -> WType {
        WType {
            reference: WReference::None,
            inner: self,
        }
    }
}

impl WType {
    pub fn into_general(self) -> WGeneralType {
        WGeneralType::Normal(self)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WPartialGeneralType {
    Unknown,
    Normal(WType),
    PanicResult(Option<WType>),
    PhiArg(Option<WType>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WReference {
    Mutable,
    Immutable,
    None,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WTypeArray {
    pub index_width: u32,
    pub element_width: u32,
}

impl IntoSyn<Type> for WBasicType {
    fn into_syn(self) -> Type {
        let span = Span::call_site();
        match self {
            WBasicType::Bitvector(width) => create_mck_type("Bitvector", &[width], span),
            WBasicType::Unsigned(width) => create_mck_type("Unsigned", &[width], span),
            WBasicType::Signed(width) => create_mck_type("Signed", &[width], span),
            WBasicType::BitvectorArray(array) => create_mck_type(
                "BitvectorArray",
                &[array.index_width, array.element_width],
                span,
            ),
            WBasicType::Path(path) => Type::Path(TypePath {
                qself: None,
                path: path.into(),
            }),
            WBasicType::Boolean => Type::Path(TypePath {
                qself: None,
                path: Path {
                    leading_colon: Some(Token![::](span)),
                    segments: Punctuated::from_iter(["mck", "concr", "Boolean"].into_iter().map(
                        |name| PathSegment {
                            ident: Ident::new(name, span),
                            arguments: PathArguments::None,
                        },
                    )),
                },
            }),
        }
    }
}

impl IntoSyn<Type> for WType {
    fn into_syn(self) -> Type {
        let span = Span::call_site();

        let simple_type = self.inner.into_syn();

        match self.reference {
            WReference::Mutable => Type::Reference(TypeReference {
                and_token: Token![&](span),
                lifetime: None,
                mutability: Some(Token![mut](span)),
                elem: Box::new(simple_type),
            }),
            WReference::Immutable => Type::Reference(TypeReference {
                and_token: Token![&](span),
                lifetime: None,
                mutability: None,
                elem: Box::new(simple_type),
            }),
            WReference::None => simple_type,
        }
    }
}

impl IntoSyn<Type> for WGeneralType {
    fn into_syn(self) -> Type {
        match self {
            WGeneralType::Normal(ty) => WPartialGeneralType::Normal(ty).into_syn(),
            WGeneralType::PanicResult(ty) => WPartialGeneralType::PanicResult(Some(ty)).into_syn(),
            WGeneralType::PhiArg(ty) => WPartialGeneralType::PhiArg(Some(ty)).into_syn(),
        }
    }
}

impl IntoSyn<Type> for WPartialGeneralType {
    fn into_syn(self) -> Type {
        let span = Span::call_site();
        match self {
            WPartialGeneralType::Normal(normal) => normal.into_syn(),
            WPartialGeneralType::PanicResult(inner) => {
                let mut segments = Punctuated::from_iter(
                    ["machine_check", "internal", "PanicResult"]
                        .into_iter()
                        .map(|name| PathSegment {
                            ident: Ident::new(name, span),
                            arguments: PathArguments::None,
                        }),
                );
                if let Some(inner) = inner {
                    let inner = inner.into_syn();
                    segments[2].arguments =
                        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: Token![<](span),
                            args: Punctuated::from_iter(vec![GenericArgument::Type(inner)]),
                            gt_token: Token![>](span),
                        });
                }
                Type::Path(TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: Some(Token![::](span)),
                        segments,
                    },
                })
            }
            WPartialGeneralType::PhiArg(inner) => {
                let span = Span::call_site();
                let mut segments =
                    Punctuated::from_iter(["mck", "forward", "PhiArg"].into_iter().map(|name| {
                        PathSegment {
                            ident: Ident::new(name, span),
                            arguments: PathArguments::None,
                        }
                    }));
                if let Some(inner) = inner {
                    let inner = inner.into_syn();
                    segments[2].arguments =
                        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: Token![<](span),
                            args: Punctuated::from_iter(vec![GenericArgument::Type(inner)]),
                            gt_token: Token![>](span),
                        });
                }
                Type::Path(TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: Some(Token![::](span)),
                        segments,
                    },
                })
            }
            WPartialGeneralType::Unknown => Type::Infer(TypeInfer {
                underscore_token: Token![_](span),
            }),
        }
    }
}

fn create_mck_type(name: &str, widths: &[u32], span: Span) -> Type {
    let width_arg = if !widths.is_empty() {
        let widths = Punctuated::from_iter(widths.iter().map(|width| {
            GenericArgument::Const(Expr::Lit(ExprLit {
                attrs: Vec::new(),
                lit: Lit::Int(LitInt::new(&width.to_string(), span)),
            }))
        }));

        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: Token![<](span),
            args: widths,
            gt_token: Token![>](span),
        })
    } else {
        syn::PathArguments::None
    };

    let path = Path {
        leading_colon: Some(Token![::](span)),
        segments: Punctuated::from_iter([
            PathSegment {
                ident: Ident::new("machine_check", span),
                arguments: syn::PathArguments::None,
            },
            PathSegment {
                ident: Ident::new(name, span),
                arguments: width_arg,
            },
        ]),
    };
    Type::Path(TypePath { qself: None, path })
}
