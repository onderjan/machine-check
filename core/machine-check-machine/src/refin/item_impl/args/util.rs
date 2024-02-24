use syn::{spanned::Spanned, FnArg, Pat, Signature, Type};

use crate::{ErrorType, MachineError};

pub(crate) fn convert_type_to_path(ty: Type) -> Result<Type, MachineError> {
    match ty {
        Type::Path(_) => return Ok(ty),
        Type::Reference(ref reference) => {
            if let Type::Path(ref path) = *reference.elem {
                return Ok(Type::Path(path.clone()));
            }
        }
        _ => (),
    }
    Err(MachineError::new(
        ErrorType::BackwardInternal(String::from("Conversion to path type not supported")),
        ty.span(),
    ))
}

pub(crate) fn create_input_name_type_iter(
    sig: &Signature,
) -> impl Iterator<Item = Result<(String, Type), MachineError>> + '_ {
    sig.inputs.iter().map(|input| match input {
        FnArg::Receiver(receiver) => {
            let ty = receiver.ty.as_ref();
            Ok((String::from("self"), ty.clone()))
        }
        FnArg::Typed(typed) => {
            let ty = typed.ty.as_ref();
            let Pat::Ident(ref pat_ident) = *typed.pat else {
                return Err(MachineError::new(
                    ErrorType::BackwardInternal(String::from(
                        "Non-identifier patterns are not supported",
                    )),
                    ty.span(),
                ));
            };
            if pat_ident.by_ref.is_some()
                || pat_ident.mutability.is_some()
                || pat_ident.subpat.is_some()
            {
                return Err(MachineError::new(
                    ErrorType::BackwardInternal(String::from(
                        "Impure identifier patterns are not supported",
                    )),
                    ty.span(),
                ));
            }
            Ok((pat_ident.ident.to_string(), ty.clone()))
        }
    })
}
