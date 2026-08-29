use std::fmt::Debug;
use syn::Type;

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

    pub fn into_syn(self) -> Type {
        self.into_partial().into_syn()
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
