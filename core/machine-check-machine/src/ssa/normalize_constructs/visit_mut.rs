use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    Member,
};

impl Visit<'_> for super::Visitor {
    fn visit_generics(&mut self, generics: &syn::Generics) {
        if generics.lt_token.is_some() || generics.where_clause.is_some() {
            self.push_error("Generics", generics.span());
        }

        // delegate
        visit::visit_generics(self, generics);
    }

    fn visit_expr_struct(&mut self, expr_struct: &syn::ExprStruct) {
        if expr_struct.qself.is_some() {
            self.push_error("Quantified self", expr_struct.span());
        }
        if expr_struct.dot2_token.is_some() {
            self.push_error("Struct expressions with base", expr_struct.span());
        }

        // delegate
        visit::visit_expr_struct(self, expr_struct);
    }

    fn visit_expr_path(&mut self, expr_path: &syn::ExprPath) {
        if expr_path.qself.is_some() {
            self.push_error("Qualified self on path", expr_path.span());
        }

        // delegate
        visit::visit_expr_path(self, expr_path);
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

    fn visit_member(&mut self, member: &syn::Member) {
        if !matches!(member, Member::Named(_)) {
            self.push_error("Unnamed member", member.span());
        }

        // delegate
        visit::visit_member(self, member);
    }
}
