use proc_macro2::Span;
use std::fmt::Debug;
use syn::{Path, Token, Type, TypeInfer, TypePath, TypeReference};

use crate::wir::{WPartialArgument, WPartialPath, WPath, WSpan, WSpanned};

use super::IntoSyn;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WTypeId(pub usize);

impl IntoSyn<Type> for WTypeId {
    fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Type {
        type_fn(self)
    }
}

impl Debug for WTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}

#[derive(Clone, Hash)]
pub enum WPartialType {
    Path(WPartialPath),
    Reference(Box<WPartialType>),
    Infer(WSpan),
}

impl WPartialType {
    pub fn wir_span(&self) -> WSpan {
        match self {
            WPartialType::Path(path) => {
                if let Some(last) = path.segments.last() {
                    last.ident.wir_span()
                } else {
                    WSpan::call_site()
                }
            }
            WPartialType::Reference(inner) => inner.wir_span(),
            WPartialType::Infer(span) => *span,
        }
    }

    pub fn is_fully_inferred(&self) -> bool {
        match self {
            WPartialType::Path(path) => {
                for segment in &path.segments {
                    if let Some(generics) = &segment.generics {
                        for argument in &generics.arguments {
                            if let WPartialArgument::Infer(_) = argument {
                                return false;
                            }
                        }
                    }
                }
                true
            }
            WPartialType::Infer(_) => false,
            WPartialType::Reference(inner) => inner.is_fully_inferred(),
        }
    }

    pub fn into_total(self) -> Result<WType, ()> {
        match self {
            WPartialType::Path(path) => Ok(WType::Path(path.into_total()?)),
            WPartialType::Reference(inner) => Ok(WType::Reference(Box::new(inner.into_total()?))),
            WPartialType::Infer(_span) => Err(()),
        }
    }
}

impl From<WPartialType> for Type {
    fn from(value: WPartialType) -> Self {
        match value {
            WPartialType::Path(path) => {
                let path: Path = path.into();
                Type::Path(TypePath { qself: None, path })
            }
            WPartialType::Reference(ty) => {
                let span: Span = ty.wir_span().first();
                let elem = Box::new((*ty).into());
                Type::Reference(TypeReference {
                    and_token: Token![&](span),
                    lifetime: None,
                    mutability: None,
                    elem,
                })
            }
            WPartialType::Infer(span) => Type::Infer(TypeInfer {
                underscore_token: Token![_](span.first()),
            }),
        }
    }
}

impl Debug for WPartialType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => Debug::fmt(&path, f),
            Self::Reference(inner) => {
                write!(f, "&")?;
                Debug::fmt(inner.as_ref(), f)
            }
            Self::Infer(_span) => write!(f, "_"),
        }
    }
}

#[derive(Clone, Hash)]
pub enum WType {
    Path(WPath),
    Reference(Box<WType>),
}

impl WType {
    pub fn wir_span(&self) -> WSpan {
        match self {
            WType::Path(path) => {
                if let Some(last) = path.segments.last() {
                    last.ident.wir_span()
                } else {
                    WSpan::call_site()
                }
            }
            WType::Reference(inner) => inner.wir_span(),
        }
    }
}

impl From<WType> for Type {
    fn from(value: WType) -> Self {
        match value {
            WType::Path(path) => {
                let path: Path = path.into();
                Type::Path(TypePath { qself: None, path })
            }
            WType::Reference(ty) => {
                let span: Span = ty.wir_span().first();
                let elem = Box::new((*ty).into());
                Type::Reference(TypeReference {
                    and_token: Token![&](span),
                    lifetime: None,
                    mutability: None,
                    elem,
                })
            }
        }
    }
}

impl Debug for WType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => Debug::fmt(&path, f),
            Self::Reference(inner) => {
                write!(f, "&")?;
                Debug::fmt(inner.as_ref(), f)
            }
        }
    }
}
