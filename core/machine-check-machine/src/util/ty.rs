use syn::{Path, Type, TypePath, TypeReference};

use super::ArgType;

pub fn create_type_path(path: Path) -> Type {
    Type::Path(TypePath { qself: None, path })
}
pub fn create_type_reference(mutable: bool, ty: Type) -> Type {
    let mutability = if mutable {
        Some(Default::default())
    } else {
        None
    };
    Type::Reference(TypeReference {
        and_token: Default::default(),
        lifetime: Default::default(),
        mutability,
        elem: Box::new(ty),
    })
}
pub fn create_converted_type(arg_ty: ArgType, ty: Type) -> Type {
    match arg_ty {
        ArgType::Normal => ty,
        ArgType::Reference => create_type_reference(false, ty),
        ArgType::MutableReference => create_type_reference(true, ty),
    }
}

pub fn extract_type_path(ty: &Type) -> Option<Path> {
    if let Type::Path(path) = ty {
        Some(path.path.clone())
    } else {
        None
    }
}
