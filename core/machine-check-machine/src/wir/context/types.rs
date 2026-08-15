use proc_macro2::Span;

use crate::wir::{
    WIdent, WPartialArgument, WPartialGenerics, WPartialPath, WPartialSegment, WPartialType, WPath,
    WPathArgument, WPathGenerics, WPathSegment, WSpan, WType,
};

pub fn bitvector_type(width: Option<u32>) -> WPartialType {
    bitvector_like_type("Bitvector", width)
}

pub fn unsigned_type(width: Option<u32>) -> WPartialType {
    bitvector_like_type("Unsigned", width)
}

pub fn signed_type(width: Option<u32>) -> WPartialType {
    bitvector_like_type("Signed", width)
}

fn bitvector_like_type(name: &str, width: Option<u32>) -> WPartialType {
    let generics = width.map(|width| WPartialGenerics {
        turbofish: None,
        arguments: vec![WPartialArgument::Uint(width, WSpan::call_site())],
    });
    WPartialType::Path(WPartialPath {
        leading_colon: Some(WSpan::call_site()),
        segments: vec![
            WPartialSegment {
                ident: WIdent::new(String::from("machine_check"), Span::call_site()),
                generics: None,
            },
            WPartialSegment {
                ident: WIdent::new(String::from(name), Span::call_site()),
                generics,
            },
        ],
    })
}

pub fn bool_type() -> WPartialType {
    WPartialType::Path(WPartialPath {
        leading_colon: None,
        segments: vec![WPartialSegment {
            ident: WIdent::new(String::from("bool"), Span::call_site()),
            generics: None,
        }],
    })
}

pub fn phi_arg_type_path(span: WSpan, inner: Option<WType>) -> WPath {
    let generics = inner.map(|inner| WPathGenerics {
        turbofish: Some(span),
        arguments: vec![WPathArgument::Type(inner)],
    });
    WPath {
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
                generics,
            },
        ],
    }
}

pub fn phi_arg_item_path(item: String, span: WSpan) -> WPartialPath {
    WPartialPath {
        leading_colon: Some(span),
        segments: vec![
            WPartialSegment {
                ident: WIdent::new(String::from("mck"), span.first()),
                generics: None,
            },
            WPartialSegment {
                ident: WIdent::new(String::from("forward"), span.first()),
                generics: None,
            },
            WPartialSegment {
                ident: WIdent::new(String::from("PhiArg"), span.first()),
                generics: None,
            },
            WPartialSegment {
                ident: WIdent::new(item, span.first()),
                generics: None,
            },
        ],
    }
}
