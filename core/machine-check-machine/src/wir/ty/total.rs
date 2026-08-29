use std::fmt::Debug;
use syn::Type;

use crate::wir::{
    WIdent, WPartialType, WSpan, WTotalPath, WTotalPathArgument, WTotalPathGenerics,
    WTotalPathSegment,
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum WTotalType {
    Path(WTotalPath),
    Reference(Box<WTotalType>),
}

impl WTotalType {
    pub fn span(&self) -> WSpan {
        match self {
            WTotalType::Path(path) => {
                if let Some(last) = path.segments.last() {
                    last.ident.span()
                } else {
                    WSpan::call_site()
                }
            }
            WTotalType::Reference(inner) => inner.span(),
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

    pub fn new_bitvector(width: Option<u32>) -> Self {
        Self::new_bitvector_like("Bitvector", width)
    }

    pub fn new_unsigned(width: Option<u32>) -> Self {
        Self::new_bitvector_like("Unsigned", width)
    }

    pub fn new_signed(width: Option<u32>) -> Self {
        Self::new_bitvector_like("Signed", width)
    }

    fn new_bitvector_like(name: &str, width: Option<u32>) -> Self {
        let generics = width.map(|width| WTotalPathGenerics {
            turbofish: None,
            arguments: vec![WTotalPathArgument::Uint(width, WSpan::call_site())],
        });
        WTotalType::Path(WTotalPath {
            leading_colon: Some(WSpan::call_site()),
            segments: vec![
                WTotalPathSegment {
                    ident: WIdent::new(String::from("machine_check"), WSpan::call_site()),
                    generics: None,
                },
                WTotalPathSegment {
                    ident: WIdent::new(String::from(name), WSpan::call_site()),
                    generics,
                },
            ],
        })
    }

    pub fn new_bool() -> Self {
        WTotalType::Path(WTotalPath {
            leading_colon: None,
            segments: vec![WTotalPathSegment {
                ident: WIdent::new(String::from("bool"), WSpan::call_site()),
                generics: None,
            }],
        })
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
