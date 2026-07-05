use syn::{AngleBracketedGenericArguments, Expr, GenericArgument, Lit, PathArguments, Type};

use crate::{
    into_wir::Error,
    wir::{WIdent, WPartialArgument, WPartialPath, WPartialSegment, WPartialType, WSpan},
};

pub fn fold_type(ty: Type) -> Result<WPartialType, Error> {
    let ty_span = WSpan::from_syn(&ty);
    match ty {
        Type::Path(type_path) => {
            let leading_colon = type_path.path.leading_colon.map(|c| WSpan::from_syn(&c));
            let mut segments = Vec::new();
            for segment in type_path.path.segments.into_iter() {
                if segment.ident == "super" || segment.ident == "crate" || segment.ident == "$crate"
                {
                    return Err(Error::unsupported_construct(
                        "Path segment super / crate / $crate",
                        WSpan::from_span(segment.ident.span()),
                    ));
                }

                let generics = match segment.arguments {
                    PathArguments::None => None,
                    PathArguments::AngleBracketed(arguments) => {
                        Some(fold_type_arguments(arguments)?)
                    }
                    PathArguments::Parenthesized(_) => {
                        return Err(Error::unsupported_construct("Function generics", ty_span))
                    }
                };

                segments.push(WPartialSegment {
                    ident: WIdent::from_syn_ident(segment.ident),
                    generics,
                })
            }

            Ok(WPartialType::Path(WPartialPath {
                leading_colon,
                segments,
            }))
        }
        Type::Reference(type_reference) => {
            if type_reference.lifetime.is_some() {
                return Err(Error::unsupported_construct(
                    "Reference with lifetime",
                    ty_span,
                ));
            }
            if type_reference.mutability.is_some() {
                return Err(Error::unsupported_construct(
                    "Reference with mutability",
                    ty_span,
                ));
            }
            let inner = fold_type(*type_reference.elem)?;
            Ok(WPartialType::Reference(Box::new(inner)))
        }
        _ => Err(Error::unsupported_construct("Type", ty_span)),
    }
}

fn fold_type_arguments(
    arguments: AngleBracketedGenericArguments,
) -> Result<Vec<WPartialArgument>, Error> {
    if arguments.colon2_token.is_some() {
        return Err(Error::unsupported_construct(
            "Turbofish",
            WSpan::from_syn(&arguments.colon2_token),
        ));
    }
    let mut result = Vec::new();
    for argument in arguments.args {
        let arg_span = WSpan::from_syn(&argument);
        let mut arg_result = None;
        match argument {
            GenericArgument::Const(expr) => match expr {
                Expr::Lit(expr_lit) => match expr_lit.lit {
                    Lit::Int(lit_int) => {
                        let Ok(num) = lit_int.base10_parse::<u32>() else {
                            return Err(Error::unsupported_construct(
                                "Const generic argument not fitting in u32",
                                arg_span,
                            ));
                        };
                        arg_result = Some(WPartialArgument::Uint(num, WSpan::from_syn(&lit_int)));
                    }
                    _ => {
                        return Err(Error::unsupported_construct(
                            "Non-integer const generic argument",
                            arg_span,
                        ))
                    }
                },
                Expr::Infer(infer) => {
                    arg_result = Some(WPartialArgument::Infer(WSpan::from_syn(&infer)));
                }
                _ => {
                    return Err(Error::unsupported_construct(
                        "Non-literal const generic argument",
                        arg_span,
                    ))
                }
            },
            GenericArgument::Type(Type::Infer(infer)) => {
                arg_result = Some(WPartialArgument::Infer(WSpan::from_syn(&infer)));
            }
            _ => {}
        }

        if let Some(arg_result) = arg_result {
            result.push(arg_result);
        } else {
            return Err(Error::unsupported_construct(
                "Generic argument that is not const or wildcard",
                arg_span,
            ));
        }
    }
    Ok(result)
}
