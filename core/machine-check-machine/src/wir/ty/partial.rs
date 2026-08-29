use proc_macro2::Span;
use std::fmt::Debug;
use syn::{Path, Token, Type, TypeInfer, TypePath, TypeReference};

use crate::wir::{WPartialPath, WPartialPathArgument, WSpan, WSpanned, WTotalType};

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

    pub fn set_span(&mut self, new_span: WSpan) {
        match self {
            WPartialType::Path(path) => path.set_span(new_span),
            WPartialType::Reference(inner) => inner.set_span(new_span),
            WPartialType::Infer(span) => *span = new_span,
        }
    }

    pub fn is_fully_inferred(&self) -> bool {
        match self {
            WPartialType::Path(path) => {
                for segment in &path.segments {
                    if let Some(generics) = &segment.generics {
                        for argument in &generics.arguments {
                            if let WPartialPathArgument::Infer(_) = argument {
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

    pub fn into_total(self) -> Result<WTotalType, ()> {
        match self {
            WPartialType::Path(path) => Ok(WTotalType::Path(path.into_total()?)),
            WPartialType::Reference(inner) => {
                Ok(WTotalType::Reference(Box::new(inner.into_total()?)))
            }
            WPartialType::Infer(_span) => Err(()),
        }
    }

    pub fn into_syn(self) -> Type {
        match self {
            WPartialType::Path(path) => {
                let path: Path = path.into();
                Type::Path(TypePath { qself: None, path })
            }
            WPartialType::Reference(ty) => {
                let span: Span = ty.wir_span().first();
                let elem = Box::new(ty.into_syn());
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
