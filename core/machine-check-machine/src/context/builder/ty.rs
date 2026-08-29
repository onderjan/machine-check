use syn::Type;

use crate::{
    context::builder::path::fold_partial_path,
    into_wir::Error,
    wir::{WPartialType, WSpan, WTotalType},
};

pub fn fold_partial_type(ty: Type) -> Result<WPartialType, Error> {
    let ty_span = WSpan::from_syn(&ty);
    match ty {
        Type::Path(type_path) => fold_partial_path(type_path.path).map(WPartialType::Path),
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
            let inner = fold_partial_type(*type_reference.elem)?;
            Ok(WPartialType::Reference(Box::new(inner)))
        }
        _ => Err(Error::unsupported_construct("Type", ty_span)),
    }
}

pub fn fold_total_type(ty: Type) -> Result<WTotalType, Error> {
    let ty = fold_partial_type(ty)?;
    let span = ty.span();
    match ty.try_into_total() {
        Ok(ty) => Ok(ty),
        Err(()) => Err(Error::new(
            crate::into_wir::ErrorType::IllegalConstruct(String::from(
                "Interference not allowed here",
            )),
            span,
        )),
    }
}
