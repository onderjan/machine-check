use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

impl Visit<'_> for super::Visitor {
    fn visit_generics(&mut self, generics: &syn::Generics) {
        if generics.lt_token.is_some() || generics.where_clause.is_some() {
            self.push_error("Generics", generics.span());
        }

        // delegate
        visit::visit_generics(self, generics);
    }

    fn visit_type(&mut self, ty: &syn::Type) {
        match ty {
            syn::Type::Path(_) => {
                // OK
            }
            syn::Type::Reference(ty_ref) => {
                if ty_ref.mutability.is_some() {
                    self.push_error("Mutable reference", ty_ref.span());
                }
                if ty_ref.lifetime.is_some() {
                    self.push_error("Lifetime", ty_ref.span());
                }
                if !matches!(*ty_ref.elem, syn::Type::Path(_)) {
                    self.push_error(
                        "Type that is not path or single-reference path",
                        ty_ref.span(),
                    );
                }
            }
            _ => {
                self.push_error("Type that is not path or single-reference path", ty.span());
            }
        }

        // delegate
        visit::visit_type(self, ty);
    }
}
