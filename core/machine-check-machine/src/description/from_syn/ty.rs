use syn::{spanned::Spanned, Expr, GenericArgument, PathArguments, Type};

use crate::{
    description::{from_syn::path::fold_path, ErrorType, Error},
    wir::{WBasicType, WReference, WType, WTypeArray},
};

pub fn fold_type(mut ty: Type) -> Result<WType<WBasicType>, Error> {
    let reference = match ty {
        Type::Reference(type_reference) => {
            if type_reference.mutability.is_some() {
                return Err(Error::unsupported_construct(
                    "Mutable references",
                    type_reference.mutability.span(),
                ));
            }
            ty = *type_reference.elem;
            WReference::Immutable
        }
        _ => WReference::None,
    };
    Ok(WType {
        reference,
        inner: fold_basic_type(ty)?,
    })
}

pub fn fold_basic_type(ty: Type) -> Result<WBasicType, Error> {
    let ty_span = ty.span();
    match ty {
        Type::Path(ty) => {
            if ty.qself.is_some() {
                return Err(Error::unsupported_construct("Quantified self", ty.span()));
            }

            let mut known_type = None;
            if ty.path.leading_colon.is_some() && !ty.path.segments.is_empty() {
                let mut segments_iter = ty.path.segments.clone().into_pairs();
                let first_segment = segments_iter.next().unwrap().into_value();

                if &first_segment.ident.to_string() == "machine_check"
                    && ty.path.segments.len() >= 2
                {
                    let second_segment = segments_iter.next().unwrap().into_value();
                    let arguments = second_segment.arguments;

                    if ty.path.segments.len() == 2 {
                        known_type = match second_segment.ident.to_string().as_str() {
                            "Bitvector" => Some(WBasicType::Bitvector(
                                extract_generic_sizes(arguments, 1)?[0],
                            )),
                            "Unsigned" => Some(WBasicType::Unsigned(
                                extract_generic_sizes(arguments, 1)?[0],
                            )),
                            "Signed" => {
                                Some(WBasicType::Signed(extract_generic_sizes(arguments, 1)?[0]))
                            }
                            "BitvectorArray" => {
                                let sizes = extract_generic_sizes(arguments, 2)?;
                                Some(WBasicType::BitvectorArray(WTypeArray {
                                    index_width: sizes[0],
                                    element_width: sizes[1],
                                }))
                            }
                            _ => {
                                return Err(Error::new(
                                    ErrorType::IllegalConstruct(String::from(
                                        "Unknown machine-check type",
                                    )),
                                    ty_span,
                                ))
                            }
                        };
                    }
                }
            }

            Ok(if let Some(known_type) = known_type {
                known_type
            } else {
                WBasicType::Path(fold_path(ty.path)?)
            })
        }
        _ => Err(Error::unsupported_construct("Non-path type", ty_span)),
    }
}

pub fn extract_generic_sizes(
    arguments: PathArguments,
    expected_length: usize,
) -> Result<Vec<u32>, Error> {
    let mut generic_sizes = Vec::new();
    match arguments {
        syn::PathArguments::None => {}
        syn::PathArguments::AngleBracketed(generic_args) => {
            assert_eq!(expected_length, generic_args.args.len());
            for arg in generic_args.args.into_iter() {
                let arg_span = arg.span();
                let parsed = match arg {
                    GenericArgument::Const(Expr::Lit(expr)) => match expr.lit {
                        syn::Lit::Int(lit_int) => {
                            let value: Result<u32, _> = lit_int.base10_parse();
                            value.ok()
                        }
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(parsed) = parsed {
                    generic_sizes.push(parsed);
                } else {
                    return Err(Error::unsupported_construct(
                        "Generic argument not parseable as u32",
                        arg_span,
                    ));
                }
            }
        }
        syn::PathArguments::Parenthesized(_) => {
            return Err(Error::unsupported_construct(
                "Parenthesized",
                arguments.span(),
            ));
        }
    };
    Ok(generic_sizes)
}
