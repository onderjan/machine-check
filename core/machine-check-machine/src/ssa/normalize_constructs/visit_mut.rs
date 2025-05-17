use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    Expr, Member, Pat,
};

impl Visit<'_> for super::Visitor {
    fn visit_generics(&mut self, generics: &syn::Generics) {
        if generics.lt_token.is_some() || generics.where_clause.is_some() {
            self.push_error("Generics", generics.span());
        }

        // delegate
        visit::visit_generics(self, generics);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        // delegate first to avoid spurious path errors
        visit::visit_expr(self, expr);

        match expr {
            Expr::Block(_)
            | Expr::Assign(_)
            | Expr::Index(_)
            | Expr::Group(_)
            | Expr::Paren(_)
            | Expr::Reference(_)
            | Expr::Lit(_)
            | Expr::Field(_)
            | Expr::Struct(_)
            | Expr::Call(_)
            | Expr::Path(_)
            | Expr::If(_)
            | Expr::Unary(_)
            | Expr::Binary(_)
            | Expr::Macro(_) => {}
            _ => {
                self.push_error("Expression type", expr.span());
            }
        }
    }

    fn visit_expr_call(&mut self, expr_call: &syn::ExprCall) {
        if !matches!(*expr_call.func, Expr::Path(_)) {
            self.push_error("Non-path function operand", expr_call.span());
        }

        // delegate
        visit::visit_expr_call(self, expr_call);
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

    fn visit_pat(&mut self, pat: &Pat) {
        match pat {
            Pat::Ident(_) | Pat::Lit(_) | Pat::Type(_) => {}
            _ => self.push_error("Pattern type", pat.span()),
        };

        // delegate
        visit::visit_pat(self, pat);
    }

    fn visit_pat_ident(&mut self, pat_ident: &syn::PatIdent) {
        if pat_ident.by_ref.is_some() {
            self.push_error("Pattern binding to reference", pat_ident.by_ref.span());
        }

        if let Some((_at, subpat)) = &pat_ident.subpat {
            self.push_error("Subpattern", subpat.span());
        }

        // delegate
        visit::visit_pat_ident(self, pat_ident);
    }

    fn visit_pat_type(&mut self, pat_type: &syn::PatType) {
        if !matches!(*pat_type.pat, Pat::Ident(_)) {
            self.push_error("Pattern other than ident or typed ident", pat_type.span());
        }

        // delegate
        visit::visit_pat_type(self, pat_type);
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
