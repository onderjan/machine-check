use proc_macro2::Span;

use crate::wir::{
    WIdent, WPartialPath, WPartialPathArgument, WPartialPathGenerics, WPartialPathSegment,
    WPartialType, WSpan,
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
    let generics = width.map(|width| WPartialPathGenerics {
        turbofish: None,
        arguments: vec![WPartialPathArgument::Uint(width, WSpan::call_site())],
    });
    WPartialType::Path(WPartialPath {
        leading_colon: Some(WSpan::call_site()),
        segments: vec![
            WPartialPathSegment {
                ident: WIdent::new(String::from("machine_check"), Span::call_site()),
                generics: None,
            },
            WPartialPathSegment {
                ident: WIdent::new(String::from(name), Span::call_site()),
                generics,
            },
        ],
    })
}

pub fn bool_type() -> WPartialType {
    WPartialType::Path(WPartialPath {
        leading_colon: None,
        segments: vec![WPartialPathSegment {
            ident: WIdent::new(String::from("bool"), Span::call_site()),
            generics: None,
        }],
    })
}
