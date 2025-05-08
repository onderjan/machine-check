use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, Expr, ExprLit, GenericArgument, Ident, Lit, LitInt, Path,
    PathArguments, PathSegment, Token, Type, TypePath, TypeReference,
};

use super::{IntoSyn, WPath};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WSimpleType {
    Bitvector(u32),
    BitvectorArray(WTypeArray),
    Unsigned(u32),
    Signed(u32),
    Boolean,
    Path(WPath),
}

impl WSimpleType {
    pub fn into_type(self) -> WType {
        WType {
            reference: WReference::None,
            inner: self,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WType {
    pub reference: WReference,
    pub inner: WSimpleType,
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

impl IntoSyn<Type> for WSimpleType {
    fn into_syn(self) -> Type {
        let span = Span::call_site();
        match self {
            WSimpleType::Bitvector(width) => create_mck_type("Bitvector", &[width], span),
            WSimpleType::Unsigned(width) => create_mck_type("Unsigned", &[width], span),
            WSimpleType::Signed(width) => create_mck_type("Signed", &[width], span),
            WSimpleType::BitvectorArray(array) => create_mck_type(
                "BitvectorArray",
                &[array.index_width, array.element_width],
                span,
            ),
            WSimpleType::Path(path) => Type::Path(TypePath {
                qself: None,
                path: path.into(),
            }),
            WSimpleType::Boolean => Type::Path(TypePath {
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
