use proc_macro2::Span;
use std::fmt::Debug;
use syn::{Path, Token, Type, TypePath, TypeReference};

use crate::wir::{WPartialType, WSpan, WSpanned, WTotalPath};

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum WTotalType {
    Path(WTotalPath),
    Reference(Box<WTotalType>),
}

impl WTotalType {
    pub fn wir_span(&self) -> WSpan {
        match self {
            WTotalType::Path(path) => {
                if let Some(last) = path.segments.last() {
                    last.ident.wir_span()
                } else {
                    WSpan::call_site()
                }
            }
            WTotalType::Reference(inner) => inner.wir_span(),
        }
    }

    pub fn into_partial(self) -> WPartialType {
        match self {
            WTotalType::Path(path) => WPartialType::Path(path.into_partial()),
            WTotalType::Reference(ty) => WPartialType::Reference(Box::new(ty.into_partial())),
        }
    }
}

impl From<WTotalType> for Type {
    fn from(value: WTotalType) -> Self {
        match value {
            WTotalType::Path(path) => {
                let path: Path = path.into();
                Type::Path(TypePath { qself: None, path })
            }
            WTotalType::Reference(ty) => {
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

impl Debug for WTotalType {
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
