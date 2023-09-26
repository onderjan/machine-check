use anyhow::anyhow;
use proc_macro2::Span;
use syn::{visit_mut::VisitMut, Ident, ItemStruct, Path, PathSegment, Type, TypePath};

pub struct TypePathVisitor {
    first_error: Option<anyhow::Error>,
}

impl TypePathVisitor {
    pub fn new() -> TypePathVisitor {
        TypePathVisitor { first_error: None }
    }

    pub fn visit_struct(&mut self, s: &mut ItemStruct) {
        self.visit_item_struct_mut(s);
    }

    pub fn transcribe_path(path: &mut Path) -> Result<(), anyhow::Error> {
        // only transcribe paths that start with leading colon
        if path.leading_colon.is_none() {
            return Ok(());
        }
        let mut segments_mut = path.segments.iter_mut();
        let Some(crate_segment) = segments_mut.next() else {
            return Ok(());
        };
        // only transcribe mck crate paths
        if crate_segment.ident != "mck" {
            return Ok(());
        }
        let Some(type_segment) = segments_mut.next() else {
            return Ok(());
        };
        let transcribed_type = match type_segment.ident.to_string().as_str() {
            "ThreeValuedArray" => Some("MarkArray"),
            "ThreeValuedBitvector" => Some("MarkBitvector"),
            _ => None,
        };
        // replace the type segment identifier
        if let Some(transcribed_type) = transcribed_type {
            type_segment.ident = syn::Ident::new(transcribed_type, type_segment.ident.span());
        }
        Ok(())
    }
}

impl VisitMut for TypePathVisitor {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if let Err(err) = Self::transcribe_path(path) {
            if self.first_error.is_none() {
                self.first_error = Some(err);
            }
        }
        // delegate
        syn::visit_mut::visit_path_mut(self, path);
    }
}
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
