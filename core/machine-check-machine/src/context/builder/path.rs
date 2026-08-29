use syn::{AngleBracketedGenericArguments, Expr, GenericArgument, Lit, Path, PathArguments, Type};

use crate::{
    context::builder::ty::fold_partial_type,
    into_wir::Error,
    wir::{
        WIdent, WPartialPath, WPartialPathArgument, WPartialPathGenerics, WPartialPathSegment,
        WSpan,
    },
};

pub fn fold_partial_path(path: Path) -> Result<WPartialPath, Error> {
    let leading_colon = path.leading_colon.map(|c| WSpan::from_syn(&c));
    let mut segments = Vec::new();
    for segment in path.segments.into_iter() {
        if segment.ident == "super" || segment.ident == "crate" || segment.ident == "$crate" {
            return Err(Error::unsupported_construct(
                "Path segment super / crate / $crate",
                WSpan::from_span(segment.ident.span()),
            ));
        }

        let generics = match segment.arguments {
            PathArguments::None => None,
            PathArguments::AngleBracketed(arguments) => {
                Some(fold_partial_path_arguments(arguments)?)
            }
            PathArguments::Parenthesized(parenthesized) => {
                return Err(Error::unsupported_construct(
                    "Function generics",
                    WSpan::from_syn(&parenthesized),
                ))
            }
        };

        segments.push(WPartialPathSegment {
            ident: WIdent::from_syn_ident(segment.ident),
            generics,
        })
    }

    Ok(WPartialPath {
        leading_colon,
        segments,
    })
}

fn fold_partial_path_arguments(
    generics: AngleBracketedGenericArguments,
) -> Result<WPartialPathGenerics, Error> {
    let turbofish = generics
        .colon2_token
        .map(|turbofish| WSpan::from_syn(&turbofish));
    let mut arguments: Vec<WPartialPathArgument> = Vec::new();
    for argument in generics.args {
        let arg_span = WSpan::from_syn(&argument);
        let arg_result = match argument {
            GenericArgument::Const(expr) => match expr {
                Expr::Lit(expr_lit) => match expr_lit.lit {
                    Lit::Int(lit_int) => {
                        let Ok(num) = lit_int.base10_parse::<u32>() else {
                            return Err(Error::unsupported_construct(
                                "Const generic argument not fitting in u32",
                                arg_span,
                            ));
                        };
                        WPartialPathArgument::Uint(num, WSpan::from_syn(&lit_int))
                    }
                    _ => {
                        return Err(Error::unsupported_construct(
                            "Non-integer const generic argument",
                            arg_span,
                        ))
                    }
                },
                Expr::Infer(infer) => WPartialPathArgument::Infer(WSpan::from_syn(&infer)),
                _ => {
                    return Err(Error::unsupported_construct(
                        "Non-literal const generic argument",
                        arg_span,
                    ))
                }
            },
            GenericArgument::Type(Type::Infer(infer)) => {
                WPartialPathArgument::Infer(WSpan::from_syn(&infer))
            }
            GenericArgument::Type(ty) => {
                let ty = fold_partial_type(ty)?;
                WPartialPathArgument::Type(ty)
            }
            _ => {
                return Err(Error::unsupported_construct(
                    "Type of generic argument",
                    arg_span,
                ))
            }
        };

        arguments.push(arg_result);
    }
    Ok(WPartialPathGenerics {
        turbofish,
        arguments,
    })
}
