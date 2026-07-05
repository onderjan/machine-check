use syn::Type;

use crate::{
    into_wir::{from_syn::path::fold_partial_path, Error},
    wir::{WPartialType, WSpan},
};

pub fn fold_type(ty: Type) -> Result<WPartialType, Error> {
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
            let inner = fold_type(*type_reference.elem)?;
            Ok(WPartialType::Reference(Box::new(inner)))
        }
        _ => Err(Error::unsupported_construct("Type", ty_span)),
    }
}
