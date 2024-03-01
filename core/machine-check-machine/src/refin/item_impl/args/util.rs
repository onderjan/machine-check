use syn::{spanned::Spanned, FnArg, Pat, Signature, Type};

use crate::{BackwardError, BackwardErrorType};

pub(crate) fn create_input_name_type_iter(
    sig: &Signature,
) -> impl Iterator<Item = Result<(String, Type), BackwardError>> + '_ {
    sig.inputs.iter().map(|input| match input {
        FnArg::Receiver(receiver) => {
            let ty = receiver.ty.as_ref();
            Ok((String::from("self"), ty.clone()))
        }
        FnArg::Typed(typed) => {
            let ty = typed.ty.as_ref();
            let Pat::Ident(ref pat_ident) = *typed.pat else {
                return Err(BackwardError::new(
                    BackwardErrorType::UnsupportedConstruct(String::from(
                        "Non-identifier patterns are not supported",
                    )),
                    ty.span(),
                ));
            };
            if pat_ident.by_ref.is_some()
                || pat_ident.mutability.is_some()
                || pat_ident.subpat.is_some()
            {
                return Err(BackwardError::new(
                    BackwardErrorType::UnsupportedConstruct(String::from(
                        "Impure identifier patterns are not supported",
                    )),
                    ty.span(),
                ));
            }
            Ok((pat_ident.ident.to_string(), ty.clone()))
        }
    })
}
