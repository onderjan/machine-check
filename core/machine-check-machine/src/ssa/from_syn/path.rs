use syn::{Expr, GenericArgument, Lit, Path, PathArguments};

use crate::{
    wir::{WBasicType, WGeneric, WGenerics, WIdent, WPath, WPathSegment},
    MachineError,
};

use super::ty::fold_type;

pub fn fold_global_path(path: Path) -> Result<WPath<WBasicType>, MachineError> {
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
                                Err(err) => {
                                    panic!("Could not parse const generic argument: {}", err)
                                }
                            };
                            WGeneric::Const(parsed)
                        }
                        _ => panic!("Unexpected type of generic argument"),
                    });
                }
                Some(WGenerics {
                    leading_colon: generics.colon2_token.is_some(),
                    inner: generic_arguments,
                })
            }

            PathArguments::Parenthesized(_generics) => {
                panic!("Unexpected parenthesized generic arguments")
            }
        };

        segments.push(WPathSegment {
            ident: WIdent::from_syn_ident(segment.ident),
            generics,
        });
    }

    Ok(WPath {
        leading_colon: path.leading_colon.is_some(),
        segments,
    })
}
