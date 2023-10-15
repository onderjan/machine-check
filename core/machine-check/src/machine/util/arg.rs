use syn::{FnArg, Ident, Pat, PatType, Receiver, Type};
use syn_path::path;

use super::{create_converted_type, create_pat_ident, create_type_path, create_type_reference};

#[derive(Clone)]
pub enum ArgType {
    Normal,
    Reference,
    MutableReference,
}

pub fn create_self_arg(arg_ty: ArgType) -> FnArg {
    let ty = create_type_path(path!(Self));
    let (reference, mutability, ty) = match arg_ty {
        ArgType::Normal => (None, None, ty),
        ArgType::Reference => (
            Some((Default::default(), None)),
            None,
            create_type_reference(false, ty),
        ),
        ArgType::MutableReference => (
            Some(Default::default()),
            Some(Default::default()),
            create_type_reference(true, ty),
        ),
    };
    FnArg::Receiver(Receiver {
        attrs: vec![],
        reference,
        mutability,
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(ty),
    })
}

pub fn create_arg(arg_ty: ArgType, ident: Ident, ty: Option<Type>) -> FnArg {
    let ty = match ty {
        Some(ty) => ty,
        None => create_type_path(path!(Self)),
    };

    let ty = create_converted_type(arg_ty, ty);
    FnArg::Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(create_pat_ident(ident))),
        colon_token: Default::default(),
        ty: Box::new(ty),
    })
}
