use syn::{spanned::Spanned, visit::Visit};

use super::error::{DescriptionError, DescriptionErrors};

pub struct AttributeDisallower {
    errors: Vec<DescriptionError>,
}

impl AttributeDisallower {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn into_result(self) -> Result<(), DescriptionErrors> {
        DescriptionErrors::iter_to_result(self.errors)
    }
}

impl Visit<'_> for AttributeDisallower {
    fn visit_attribute(&mut self, attribute: &syn::Attribute) {
        if let syn::Meta::NameValue(meta) = &attribute.meta {
            if meta.path.is_ident("doc") {
                // doc comments are allowed everywhere
                return;
            }
        }

        self.errors.push(DescriptionError::unsupported_construct(
            "Attribute here",
            attribute.span(),
        ));
    }
}
