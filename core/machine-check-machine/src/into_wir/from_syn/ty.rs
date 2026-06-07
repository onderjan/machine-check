use machine_check_common::{
    ir_common::{IrReference, IrTypeArray},
    Signedness,
};
use syn::{Expr, GenericArgument, PathArguments, Type};

use crate::{
    into_wir::{from_syn::path::fold_path, Error, ErrorType},
    wir::{WContext, WPath, WSpan, WTypeId},
};

pub fn fold_type(
    ctx: &mut WContext,
    mut ty: Type,
    self_ty: Option<&WPath>,
) -> Result<WTypeId, Error> {
    todo!("Fold type")
    /*let reference = match ty {
        Type::Reference(type_reference) => {
            if type_reference.mutability.is_some() {
                return Err(Error::unsupported_syn_construct(
                    "Mutable references",
                    &type_reference.mutability,
                ));
            }
            ty = *type_reference.elem;
            IrReference::Immutable
        }
        _ => IrReference::None,
    };
    Ok(WType {
        reference,
        inner: fold_basic_type(ty, self_ty)?,
    })*/
}
/*
pub fn fold_basic_type(ty: Type, self_ty: Option<&WPath>) -> Result<WPartialBasicType, Error> {
    let ty_span = WSpan::from_syn(&ty);
    let ty = match ty {
        Type::Path(ty) => ty,
        _ => return Err(Error::unsupported_syn_construct("Non-path type", &ty)),
    };

    if ty.qself.is_some() {
        return Err(Error::unsupported_syn_construct("Quantified self", &ty));
    }

    let mut known_type = None;
    if ty.path.leading_colon.is_some() && !ty.path.segments.is_empty() {
        let mut segments_iter = ty.path.segments.clone().into_pairs();
        let first_segment = segments_iter.next().unwrap().into_value();

        if &first_segment.ident.to_string() == "machine_check" && ty.path.segments.len() >= 2 {
            let second_segment = segments_iter.next().unwrap().into_value();
            let arguments = second_segment.arguments;

            if ty.path.segments.len() == 2 {
                known_type = match second_segment.ident.to_string().as_str() {
                    "Bitvector" => Some(WPartialBasicType::Bitvector(
                        Signedness::None,
                        extract_generic_size_opt(arguments)?,
                    )),
                    "Unsigned" => Some(WPartialBasicType::Bitvector(
                        Signedness::Unsigned,
                        extract_generic_size_opt(arguments)?,
                    )),
                    "Signed" => Some(WPartialBasicType::Bitvector(
                        Signedness::Signed,
                        extract_generic_size_opt(arguments)?,
                    )),
                    "BitvectorArray" => {
                        let sizes = extract_generic_sizes(arguments, 2)?;
                        Some(WPartialBasicType::BitvectorArray(IrTypeArray {
                            index_width: sizes[0],
                            element_width: sizes[1],
                        }))
                    }
                    _ => {
                        return Err(Error::new(
                            ErrorType::IllegalConstruct(String::from("Unknown machine-check type")),
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
        WPartialBasicType::Path(fold_path(ty.path, self_ty)?)
    })
}

pub fn extract_generic_size_opt(arguments: PathArguments) -> Result<Option<u32>, Error> {
    Ok(if matches!(arguments, syn::PathArguments::None) {
        None
    } else {
        Some(extract_generic_sizes(arguments, 1)?[0])
    })
}

pub fn extract_generic_sizes(
    arguments: PathArguments,
    expected_length: usize,
) -> Result<Vec<u32>, Error> {
    let mut generic_sizes = Vec::new();
    match arguments {
        syn::PathArguments::None => {
            if expected_length != 0 {
                return Err(Error::new(
                    ErrorType::IllegalConstruct(format!(
                        "Expected {} generic arguments but none supplied",
                        expected_length
                    )),
                    WSpan::from_syn(&arguments),
                ));
            }
        }
        syn::PathArguments::AngleBracketed(generic_args) => {
            if expected_length != generic_args.args.len() {
                return Err(Error::new(
                    ErrorType::IllegalConstruct(format!(
                        "Expected {} generic arguments, but {} supplied",
                        expected_length,
                        generic_args.args.len()
                    )),
                    WSpan::from_syn(&generic_args),
                ));
            }
            for arg in generic_args.args.into_iter() {
                let arg_span = WSpan::from_syn(&arg);
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
            return Err(Error::unsupported_syn_construct(
                "Parenthesized",
                &arguments,
            ));
        }
    };
    Ok(generic_sizes)
}
*/
