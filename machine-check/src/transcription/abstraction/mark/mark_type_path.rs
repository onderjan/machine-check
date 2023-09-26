use anyhow::anyhow;
use proc_macro2::Span;
use syn::{Ident, Path, PathSegment, Type, TypePath};

pub fn convert_type_path_to_original(path: &Path) -> Path {
    if path.leading_colon.is_some() {
        return path.clone();
    }

    let mut orig_path_segments = path.segments.clone();

    orig_path_segments.insert(
        0,
        PathSegment {
            ident: Ident::new("super", Span::call_site()),
            arguments: syn::PathArguments::None,
        },
    );

    Path {
        leading_colon: None,
        segments: orig_path_segments,
    }
}

pub fn convert_type_to_original(ty: &Type) -> anyhow::Result<Type> {
    if let Type::Reference(reference) = ty {
        let mut result = reference.clone();
        result.elem = Box::new(convert_type_to_original(&result.elem)?);
        return Ok(Type::Reference(result));
    }

    let Type::Path(TypePath{qself: None, path}) = ty else {
        return Err(anyhow!("Conversion of type {:?} to super not supported", ty));
    };

    Ok(Type::Path(TypePath {
        qself: None,
        path: convert_type_path_to_original(path),
    }))
}
