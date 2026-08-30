use std::fmt::Debug;

use crate::wir::{WPartialType, WSpan, WTypeId, WTypePath};

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum WTotalType {
    Path(WTypePath),
    Reference(WTypeId, WSpan),
    Number(u32, WSpan),
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
            WTotalType::Reference(_inner, span) => *span,
            WTotalType::Number(_num, span) => *span,
        }
    }

    pub fn into_partial(self) -> WPartialType {
        match self {
            WTotalType::Path(path) => WPartialType::Path(path),
            WTotalType::Reference(type_id, span) => WPartialType::Reference(type_id, span),
            WTotalType::Number(num, span) => WPartialType::Number(num, span),
        }
    }

    /*pub fn into_syn(self) -> Type {
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
    }*/
}

impl Debug for WTotalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => Debug::fmt(&path, f),
            Self::Reference(type_id, _span) => {
                write!(f, "&")?;
                Debug::fmt(&type_id, f)
            }
            WTotalType::Number(num, _span) => Debug::fmt(&num, f),
        }
    }
}
