use syn::{spanned::Spanned, Expr, GenericArgument, Lit, Path, PathArguments};

use crate::{
    ssa::error::DescriptionError,
    wir::{WBasicType, WGeneric, WGenerics, WIdent, WPath, WPathSegment},
};

use super::ty::fold_type;

pub fn fold_path(path: Path) -> Result<WPath<WBasicType>, DescriptionError> {
    let path_span = path.span();

    let mut segments = Vec::new();

    for segment in path.segments {
        let generics = match segment.arguments {
            PathArguments::None => None,
            PathArguments::AngleBracketed(generics) => {
                let mut generic_arguments = Vec::new();
                for arg in generics.args {
                    generic_arguments.push(match arg {
                        GenericArgument::Type(ty) => WGeneric::Type(fold_type(ty)?),
                        GenericArgument::Const(expr) => {
                            let Expr::Lit(expr) = expr else {
                                return Err(DescriptionError::unsupported_construct(
                                    "Non-literal const generic argument",
                                    path_span,
                                ));
                            };
                            let parsed: Result<u32, _> = match expr.lit {
                                Lit::Int(lit_int) => lit_int.base10_parse(),
                                _ => {
                                    return Err(DescriptionError::unsupported_construct(
                                        "Non-integer const generic argument",
                                        path_span,
                                    ));
                                }
                            };
                            let parsed = match parsed {
                                Ok(ok) => ok,
                                Err(_) => {
                                    return Err(DescriptionError::unsupported_construct(
                                        "Generic argument not parseable as u32",
                                        path_span,
                                    ));
                                }
                            };
                            WGeneric::Const(parsed)
                        }
                        _ => {
                            return Err(DescriptionError::unsupported_construct(
                                "Generic argument kind",
                                path_span,
                            ))
                        }
                    });
                }
                Some(WGenerics {
                    leading_colon: generics.colon2_token.is_some(),
                    inner: generic_arguments,
                })
            }

            PathArguments::Parenthesized(_generics) => {
                return Err(DescriptionError::unsupported_construct(
                    "Parenthesized generic arguments",
                    path_span,
                ));
            }
        };

        segments.push(WPathSegment {
            ident: WIdent::from_syn_ident(segment.ident),
            generics,
        });
    }

    // for now, disallow paths that can break out (super / crate / $crate)
    for segment in segments.iter() {
        if segment.ident.name() == "super"
            || segment.ident.name() == "crate"
            || segment.ident.name() == "$crate"
        {
            return Err(DescriptionError::unsupported_construct(
                "Path segment super / crate / $crate",
                segment.ident.span(),
            ));
        }
    }

    let has_leading_colon = path.leading_colon.is_some();

    // disallow global paths to any other crates than machine_check and std
    if has_leading_colon {
        let crate_segment = segments
            .first()
            .expect("Global path should have at least one segment");
        let crate_ident = &crate_segment.ident;
        if crate_ident.name() != "machine_check" && crate_ident.name() != "std" {
            return Err(DescriptionError::unsupported_construct(
                "Absolute paths not starting with 'machine_check' or 'std'",
                path_span,
            ));
        }
    }

    Ok(WPath {
        leading_colon: has_leading_colon,
        segments,
    })
}
