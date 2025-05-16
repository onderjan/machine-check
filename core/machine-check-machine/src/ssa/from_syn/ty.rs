use syn::{Expr, GenericArgument, PathArguments, Type};

use crate::{
    ssa::{error::DescriptionError, from_syn::path::fold_path},
    wir::{WBasicType, WPartialGeneralType, WReference, WType, WTypeArray},
};

pub fn fold_type(mut ty: Type) -> Result<WType<WBasicType>, DescriptionError> {
    let reference = match ty {
        Type::Reference(type_reference) => {
            let mutable = type_reference.mutability.is_some();
            ty = *type_reference.elem;
            if mutable {
                WReference::Mutable
            } else {
                WReference::Immutable
            }
        }
        _ => WReference::None,
    };
    Ok(WType {
        reference,
        inner: fold_basic_type(ty)?,
    })
}

pub fn fold_basic_type(ty: Type) -> Result<WBasicType, DescriptionError> {
    match ty {
        Type::Path(ty) => {
            assert!(ty.qself.is_none());

            let mut known_type = None;
            if ty.path.leading_colon.is_some() {
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
                                extract_generic_sizes(arguments, 1)[0],
                            )),
                            "Unsigned" => {
                                Some(WBasicType::Unsigned(extract_generic_sizes(arguments, 1)[0]))
                            }
                            "Signed" => {
                                Some(WBasicType::Signed(extract_generic_sizes(arguments, 1)[0]))
                            }
                            "BitvectorArray" => {
                                let sizes = extract_generic_sizes(arguments, 2);
                                Some(WBasicType::BitvectorArray(WTypeArray {
                                    index_width: sizes[0],
                                    element_width: sizes[1],
                                }))
                            }
                            _ => panic!("Unknown machine-check path type"),
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
        _ => panic!("Unexpected non-path type: {:?}", ty),
    }
}

pub fn fold_partial_general_type(
    ty: Type,
) -> Result<WPartialGeneralType<WBasicType>, DescriptionError> {
    let result: Option<_> = match &ty {
        Type::Path(ty) => {
            assert!(ty.qself.is_none());

            let mut known_type = None;
            if ty.path.leading_colon.is_some() {
                let mut segments_iter = ty.path.segments.clone().into_pairs();
                let first_segment = segments_iter.next().unwrap().into_value();

                if &first_segment.ident.to_string() == "machine_check"
                    && ty.path.segments.len() >= 2
                {
                    let second_segment = segments_iter.next().unwrap().into_value();

                    if ty.path.segments.len() == 3 {
                        let third_segment = segments_iter.next().unwrap().into_value();
                        if second_segment.ident.to_string().as_str() == "internal"
                            && third_segment.ident.to_string().as_str() == "PanicResult"
                        {
                            known_type = Some(WPartialGeneralType::PanicResult(None));
                        }
                    }
                } else if &first_segment.ident.to_string() == "mck" && ty.path.segments.len() == 3 {
                    let second_segment = segments_iter.next().unwrap().into_value();
                    let third_segment = segments_iter.next().unwrap().into_value();
                    if second_segment.ident.to_string().as_str() == "forward"
                        && third_segment.ident.to_string().as_str() == "PhiArg"
                    {
                        let mut inner_type = None;
                        if let PathArguments::AngleBracketed(generic_args) = third_segment.arguments
                        {
                            if let Some(GenericArgument::Type(inner)) = generic_args.args.first() {
                                inner_type = Some(fold_type(inner.clone())?);
                            }
                        }

                        known_type = Some(WPartialGeneralType::PhiArg(inner_type));
                    }
                }
            }
            known_type
        }
        _ => None,
    };
    Ok(if let Some(result) = result {
        result
    } else {
        WPartialGeneralType::Normal(fold_type(ty)?)
    })
}

pub fn extract_generic_sizes(arguments: PathArguments, expected_length: usize) -> Vec<u32> {
    let mut generic_sizes = Vec::new();
    match arguments {
        syn::PathArguments::None => {}
        syn::PathArguments::AngleBracketed(generic_args) => {
            assert_eq!(expected_length, generic_args.args.len());
            for arg in generic_args.args.into_iter() {
                match arg {
                    GenericArgument::Const(Expr::Lit(expr)) => match expr.lit {
                        syn::Lit::Int(lit_int) => {
                            let value: Result<u32, _> = lit_int.base10_parse();
                            let value = match value {
                                Ok(ok) => ok,
                                Err(err) => {
                                    panic!("Cannot parse generic argument: {:?}", err)
                                }
                            };
                            generic_sizes.push(value);
                        }
                        _ => panic!("Unexpected non-int generic argument"),
                    },
                    _ => panic!("Unexpected non-literal generic argument"),
                }
            }
        }
        syn::PathArguments::Parenthesized(_) => {
            panic!("Unexpected parenthesized generic arguments")
        }
    };
    generic_sizes
}
