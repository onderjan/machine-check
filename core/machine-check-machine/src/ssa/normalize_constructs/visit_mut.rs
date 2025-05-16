use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    Expr, Item, Member, Meta, Pat,
};

use crate::util::extract_path_ident;

impl Visit<'_> for super::Visitor {
    fn visit_item(&mut self, item: &syn::Item) {
        match item {
            Item::Struct(_) | Item::Impl(_) | Item::Use(_) => {}
            _ => self.push_error("Item type", item.span()),
        }

        // delegate
        visit::visit_item(self, item)
    }

    fn visit_item_struct(&mut self, item_struct: &syn::ItemStruct) {
        // handle attributes specially
        for attr in &item_struct.attrs {
            let mut is_permitted = false;
            if let Meta::List(meta_list) = &attr.meta {
                if let Some(ident) = extract_path_ident(&meta_list.path) {
                    if ident == "derive" || ident == "allow" {
                        is_permitted = true;
                    }
                }
            }
            if let Meta::NameValue(name_value) = &attr.meta {
                if let Some(ident) = extract_path_ident(&name_value.path) {
                    if ident == "doc" {
                        is_permitted = true;
                    }
                }
            }
            if !is_permitted {
                // do not mention documentation comments
                // as those are usually not written as attributes
                self.push_error(
                    "Attribute on struct that is not derive or allow",
                    attr.span(),
                );
            }
        }

        // visit other fields
        self.visit_visibility(&item_struct.vis);
        self.visit_ident(&item_struct.ident);
        self.visit_generics(&item_struct.generics);
        self.visit_fields(&item_struct.fields);
    }

    fn visit_generics(&mut self, generics: &syn::Generics) {
        if generics.lt_token.is_some() || generics.where_clause.is_some() {
            self.push_error("Generics", generics.span());
        }

        // delegate
        visit::visit_generics(self, generics);
    }

    fn visit_item_impl(&mut self, item_impl: &syn::ItemImpl) {
        if item_impl.defaultness.is_some() {
            self.push_error("Defaultness", item_impl.span());
        }
        if item_impl.unsafety.is_some() {
            self.push_error("Implementation unsafety", item_impl.span());
        }

        // delegate
        visit::visit_item_impl(self, item_impl);
    }

    fn visit_impl_item(&mut self, impl_item: &syn::ImplItem) {
        match impl_item {
            syn::ImplItem::Fn(_) | syn::ImplItem::Type(_) => {
                // OK
            }
            _ => {
                self.push_error("Item that is not function or type", impl_item.span());
            }
        }

        // delegate
        visit::visit_impl_item(self, impl_item);
    }

    fn visit_impl_item_fn(&mut self, impl_item_fn: &syn::ImplItemFn) {
        if impl_item_fn.defaultness.is_some() {
            self.push_error("Defaultness", impl_item_fn.span());
        }

        // delegate
        visit::visit_impl_item_fn(self, impl_item_fn);
    }

    fn visit_signature(&mut self, signature: &syn::Signature) {
        if signature.constness.is_some() {
            self.push_error("Constness", signature.span());
        }
        if signature.asyncness.is_some() {
            self.push_error("Asyncness", signature.span());
        }
        if signature.unsafety.is_some() {
            self.push_error("Unsafety", signature.span());
        }
        if signature.abi.is_some() {
            self.push_error("ABI", signature.span());
        }
        if signature.variadic.is_some() {
            self.push_error("Variadic argument", signature.span());
        }
        match signature.output {
            syn::ReturnType::Default => {
                self.push_error("Function without return type", signature.span());
            }
            syn::ReturnType::Type(_, _) => {}
        }

        // delegate
        visit::visit_signature(self, signature);
    }

    fn visit_receiver(&mut self, receiver: &syn::Receiver) {
        if receiver.mutability.is_some() {
            self.push_error("Mutable self parameter", receiver.mutability.span());
        }
        if let Some((_and, Some(lifetime))) = &receiver.reference {
            self.push_error("Self parameter with lifetime", lifetime.span());
        }
        // delegate
        visit::visit_receiver(self, receiver);
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

    fn visit_attribute(&mut self, attribute: &syn::Attribute) {
        let mut is_permitted = false;
        if let Meta::List(meta_list) = &attribute.meta {
            if let Some(ident) = extract_path_ident(&meta_list.path) {
                if ident == "allow" {
                    is_permitted = true;
                }
            }
        }
        if let Meta::NameValue(name_value) = &attribute.meta {
            if let Some(ident) = extract_path_ident(&name_value.path) {
                if ident == "doc" {
                    is_permitted = true;
                }
            }
        }
        if !is_permitted {
            // do not mention documentation comments
            // as those are usually not written as attributes
            self.push_error("Attributes except allow attribute", attribute.span());
        }

        // delegate
        visit::visit_attribute(self, attribute);
    }
}
