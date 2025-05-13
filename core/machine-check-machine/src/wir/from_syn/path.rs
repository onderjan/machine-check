use syn::{Expr, GenericArgument, Lit, Path, PathArguments};

use crate::wir::{WBasicType, WGeneric, WGenerics, WIdent, WPath, WPathSegment};

use super::ty::fold_type;

pub fn fold_global_path(path: Path) -> WPath<WBasicType> {
    WPath {
        leading_colon: path.leading_colon.is_some(),
        segments: path
            .segments
            .into_iter()
            .map(|path_segment| {
                let generics = match path_segment.arguments {
                    PathArguments::None => None,
                    PathArguments::AngleBracketed(generics) => Some(WGenerics {
                        leading_colon: generics.colon2_token.is_some(),
                        inner: generics
                            .args
                            .into_pairs()
                            .map(|pair| match pair.into_value() {
                                GenericArgument::Type(ty) => WGeneric::Type(fold_type(ty)),
                                GenericArgument::Const(expr) => {
                                    let Expr::Lit(expr) = expr else {
                                        panic!("Unexpected non-literal const generic argument");
                                    };
                                    let parsed: Result<u32, _> = match expr.lit {
                                        Lit::Int(lit_int) => lit_int.base10_parse(),
                                        _ => {
                                            panic!("Unexpected non-integer const generic argument")
                                        }
                                    };
                                    let parsed = match parsed {
                                        Ok(ok) => ok,
                                        Err(err) => panic!(
                                            "Could not parse const generic argument: {}",
                                            err
                                        ),
                                    };
                                    WGeneric::Const(parsed)
                                }
                                _ => panic!("Unexpected type of generic argument"),
                            })
                            .collect(),
                    }),

                    PathArguments::Parenthesized(_generics) => {
                        panic!("Unexpected parenthesized generic arguments")
                    }
                };

                WPathSegment {
                    ident: WIdent::from_syn_ident(path_segment.ident),
                    generics,
                }
            })
            .collect(),
    }
}
