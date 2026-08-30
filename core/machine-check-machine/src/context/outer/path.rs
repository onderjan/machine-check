use syn::{
    AngleBracketedGenericArguments, Expr, GenericArgument, Lit, Path, PathArguments, Type, TypePath,
};

use crate::{
    context::WOuterContext,
    wir::{
        WIdent, WPartialType, WPath, WPathGenerics, WPathSegment, WSpan, WTypeId, WTypePath,
        WTypePathSegment,
    },
    Error, ErrorType,
};

impl WOuterContext {
    pub fn fold_partial_type(&mut self, ty: Type) -> Result<WPartialType, Error> {
        let ty_span = WSpan::from_syn(&ty);
        match ty {
            Type::Path(type_path) => {
                let type_path = self.fold_type_path(type_path)?;
                Ok(WPartialType::Path(type_path))
            }
            Type::Reference(type_reference) => {
                let span = WSpan::from_syn(&type_reference);
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
                let inner = self.fold_partial_type(*type_reference.elem)?;
                let inner = self.partial_type_id(inner);
                Ok(WPartialType::Reference(inner, span))
            }
            _ => Err(Error::unsupported_construct("Type", ty_span)),
        }
    }

    pub fn fold_type_path(&mut self, type_path: TypePath) -> Result<WTypePath, Error> {
        if let Some(qself) = type_path.qself {
            return Err(Error::unsupported_construct(
                "Qualified self",
                WSpan::from_syn(&qself.ty),
            ));
        }

        let path = self.fold_partial_path(type_path.path)?;

        let mut segments = Vec::new();
        for segment in path.segments {
            let generics = if let Some(generics) = segment.generics {
                if generics.turbofish.is_some() {
                    return Err(Error::new(
                        ErrorType::IllegalConstruct(String::from("Turbofish in type")),
                        segment.ident.span(),
                    ));
                }
                Some(generics.arguments)
            } else {
                None
            };
            segments.push(WTypePathSegment {
                ident: segment.ident,
                generics,
            });
        }

        Ok(WTypePath {
            leading_colon: path.leading_colon,
            segments,
        })
    }

    pub fn fold_partial_path(&mut self, path: Path) -> Result<WPath, Error> {
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
                    Some(self.fold_partial_path_arguments(arguments)?)
                }
                PathArguments::Parenthesized(parenthesized) => {
                    return Err(Error::unsupported_construct(
                        "Function generics",
                        WSpan::from_syn(&parenthesized),
                    ))
                }
            };

            segments.push(WPathSegment {
                ident: WIdent::from_syn_ident(segment.ident),
                generics,
            })
        }

        Ok(WPath {
            leading_colon,
            segments,
        })
    }

    fn fold_partial_path_arguments(
        &mut self,
        generics: AngleBracketedGenericArguments,
    ) -> Result<WPathGenerics, Error> {
        let turbofish = generics
            .colon2_token
            .map(|turbofish| WSpan::from_syn(&turbofish));
        let mut arguments: Vec<WTypeId> = Vec::new();
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

                            WPartialType::Number(num, arg_span)
                        }
                        _ => {
                            return Err(Error::unsupported_construct(
                                "Non-integer const generic argument",
                                arg_span,
                            ))
                        }
                    },
                    Expr::Infer(infer) => WPartialType::Infer(WSpan::from_syn(&infer)),
                    _ => {
                        return Err(Error::unsupported_construct(
                            "Non-literal const generic argument",
                            arg_span,
                        ))
                    }
                },
                GenericArgument::Type(Type::Infer(infer)) => {
                    WPartialType::Infer(WSpan::from_syn(&infer))
                }
                GenericArgument::Type(ty) => self.fold_partial_type(ty)?,
                _ => {
                    return Err(Error::unsupported_construct(
                        "Type of generic argument",
                        arg_span,
                    ))
                }
            };

            arguments.push(self.partial_type_id(arg_result));
        }
        Ok(WPathGenerics {
            turbofish,
            arguments,
        })
    }
}
