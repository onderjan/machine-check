use syn::{AngleBracketedGenericArguments, Expr, GenericArgument, Lit, Path, PathArguments, Type};

use crate::{
    into_wir::Error,
    wir::{
        WIdent, WPartialArgument, WPartialGenerics, WPartialPath, WPartialSegment, WPath,
        WPathSegment, WSpan,
    },
};

pub fn fold_path(path: Path, self_ty: Option<&Type>) -> Result<WPath, Error> {
    let path_span = WSpan::from_syn(&path);

    let mut segments = Vec::new();

    for segment in path.segments {
        // TODO: add generics to WPath
        /*let PathArguments::None = segment.arguments else {
            return Err(Error::unsupported_syn_construct(
                "Generics here",
                &segment.arguments,
            ));
        };*/
        segments.push(WPathSegment {
            ident: WIdent::from_syn_ident(segment.ident),
        });
    }

    // for now, disallow paths that can break out (super / crate / $crate)
    for segment in segments.iter() {
        if segment.ident.name() == "super"
            || segment.ident.name() == "crate"
            || segment.ident.name() == "$crate"
        {
            return Err(Error::unsupported_construct(
                "Path segment super / crate / $crate",
                WSpan::from_span(segment.ident.span()),
            ));
        }
    }

    // disallow global paths to any other crates than machine_check and std
    let leading_colon = path.leading_colon.map(|leading| WSpan::from_syn(&leading));

    if leading_colon.is_some() {
        let crate_segment = segments
            .first()
            .expect("Global path should have at least one segment");
        let crate_ident = &crate_segment.ident;
        if crate_ident.name() != "machine_check" && crate_ident.name() != "std" {
            return Err(Error::unsupported_construct(
                "Absolute paths not starting with 'machine_check' or 'std'",
                path_span,
            ));
        }
    } else {
        // TODO: replace leading Self if possible
        /*
        if let Some(self_ty) = self_ty {
            if !segments.is_empty() && segments[0].ident.name() == "Self" {
                // set replaced segments spans to the original Self span
                let first_segment_span = segments[0].ident.span();
                let mut self_replacement = self_ty.clone();
                for self_ty_segment in &mut self_replacement.segments {
                    self_ty_segment.ident.set_span(first_segment_span);
                }
                // remove Self and concat
                let mut segments_iter = segments.drain(..);
                let _ = segments_iter.next();
                self_replacement.segments.extend(segments_iter);
                segments = self_replacement.segments;
                // put leading colon according to self type
                leading_colon = self_ty.leading_colon;
            }
        }*/
    }

    Ok(WPath {
        leading_colon,
        segments,
    })
}

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

        segments.push(WPartialSegment {
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
) -> Result<WPartialGenerics, Error> {
    let turbofish = generics
        .colon2_token
        .map(|turbofish| WSpan::from_syn(&turbofish));
    let mut arguments: Vec<WPartialArgument> = Vec::new();
    for argument in generics.args {
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
            arguments.push(arg_result);
        } else {
            return Err(Error::unsupported_construct(
                "Generic argument that is not const or wildcard",
                arg_span,
            ));
        }
    }
    Ok(WPartialGenerics {
        turbofish,
        arguments,
    })
}
