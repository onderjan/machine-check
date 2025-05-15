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
            _ => self.push_error(String::from("Item type not supported"), item.span()),
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
                    String::from("Only derive and allow attributes supported on structs"),
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
            self.push_error(String::from("Generics not supported"), generics.span());
        }

        // delegate
        visit::visit_generics(self, generics);
    }

    fn visit_item_impl(&mut self, item_impl: &syn::ItemImpl) {
        if item_impl.defaultness.is_some() {
            self.push_error(String::from("Defaultness not supported"), item_impl.span());
        }
        if item_impl.unsafety.is_some() {
            self.push_error(
                String::from("Implementation unsafety not supported"),
                item_impl.span(),
            );
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
                self.push_error(
                    String::from("Only functions and types supported in implementation"),
                    impl_item.span(),
                );
            }
        }

        // delegate
        visit::visit_impl_item(self, impl_item);
    }

    fn visit_impl_item_fn(&mut self, impl_item_fn: &syn::ImplItemFn) {
        if impl_item_fn.defaultness.is_some() {
            self.push_error(
                String::from("Defaultness not supported"),
                impl_item_fn.span(),
            );
        }

        // delegate
        visit::visit_impl_item_fn(self, impl_item_fn);
    }

    fn visit_signature(&mut self, signature: &syn::Signature) {
        if signature.constness.is_some() {
            self.push_error(String::from("Constness not supported"), signature.span());
        }
        if signature.asyncness.is_some() {
            self.push_error(String::from("Asyncness not supported"), signature.span());
        }
        if signature.unsafety.is_some() {
            self.push_error(String::from("Unsafety not supported"), signature.span());
        }
        if signature.abi.is_some() {
            self.push_error(String::from("ABI not supported"), signature.span());
        }
        if signature.variadic.is_some() {
            self.push_error(
                String::from("Variadic argument not supported"),
                signature.span(),
            );
        }
        match signature.output {
            syn::ReturnType::Default => {
                self.push_error(
                    String::from("Function must have return type"),
                    signature.span(),
                );
            }
            syn::ReturnType::Type(_, _) => {}
        }

        // delegate
        visit::visit_signature(self, signature);
    }

    fn visit_receiver(&mut self, receiver: &syn::Receiver) {
        if receiver.mutability.is_some() {
            self.push_error(
                String::from("Mutable self parameter not supported"),
                receiver.mutability.span(),
            );
        }
        if let Some((_and, Some(lifetime))) = &receiver.reference {
            self.push_error(
                String::from("Self parameter with lifetime not supported"),
                lifetime.span(),
            );
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
                self.push_error(String::from("Expression type not supported"), expr.span());
            }
        }
    }

    fn visit_expr_call(&mut self, expr_call: &syn::ExprCall) {
        if !matches!(*expr_call.func, Expr::Path(_)) {
            self.push_error(
                String::from("Non-path call functions not supported"),
                expr_call.span(),
            );
        }

        // delegate
        visit::visit_expr_call(self, expr_call);
    }

    fn visit_expr_struct(&mut self, expr_struct: &syn::ExprStruct) {
        if expr_struct.qself.is_some() {
            self.push_error(
                String::from("Quantified self not supported"),
                expr_struct.span(),
            );
        }
        if expr_struct.dot2_token.is_some() {
            self.push_error(
                String::from("Struct expressions with base not supported"),
                expr_struct.span(),
            );
        }

        // delegate
        visit::visit_expr_struct(self, expr_struct);
    }

    fn visit_expr_path(&mut self, expr_path: &syn::ExprPath) {
        if expr_path.qself.is_some() {
            self.push_error(
                String::from("Qualified self on path not supported"),
                expr_path.span(),
            );
        }

        // delegate
        visit::visit_expr_path(self, expr_path);
    }

    fn visit_pat(&mut self, pat: &Pat) {
        match pat {
            Pat::Ident(_) | Pat::Lit(_) | Pat::Type(_) => {}
            _ => self.push_error(String::from("Pattern type not supported"), pat.span()),
        };

        // delegate
        visit::visit_pat(self, pat);
    }

    fn visit_pat_ident(&mut self, pat_ident: &syn::PatIdent) {
        if pat_ident.by_ref.is_some() {
            self.push_error(
                String::from("Pattern binding to reference not supported"),
                pat_ident.by_ref.span(),
            );
        }

        if let Some((_at, subpat)) = &pat_ident.subpat {
            self.push_error(String::from("Subpattern not supported"), subpat.span());
        }

        // delegate
        visit::visit_pat_ident(self, pat_ident);
    }

    fn visit_pat_type(&mut self, pat_type: &syn::PatType) {
        if !matches!(*pat_type.pat, Pat::Ident(_)) {
            self.push_error(
                String::from("Non-identifier typed pattern not supported"),
                pat_type.span(),
            );
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
                    self.push_error(
                        String::from("Mutable reference not supported in type"),
                        ty_ref.span(),
                    );
                }
                if ty_ref.lifetime.is_some() {
                    self.push_error(
                        String::from("Lifetime not supported in type"),
                        ty_ref.span(),
                    );
                }
                if !matches!(*ty_ref.elem, syn::Type::Path(_)) {
                    self.push_error(
                        String::from("Only single-reference and path types are supported"),
                        ty_ref.span(),
                    );
                }
            }
            _ => {
                self.push_error(
                    String::from("Only single-reference and path types are supported"),
                    ty.span(),
                );
            }
        }

        // delegate
        visit::visit_type(self, ty);
    }

    fn visit_member(&mut self, member: &syn::Member) {
        if !matches!(member, Member::Named(_)) {
            self.push_error(
                String::from("Only named members are supported"),
                member.span(),
            );
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
            self.push_error(
                String::from("Only allow attribute supported in this context"),
                attribute.span(),
            );
        }

        // delegate
        visit::visit_attribute(self, attribute);
    }

    fn visit_path(&mut self, path: &syn::Path) {
        // for now, disallow paths that can break out (super / crate / $crate)
        for segment in path.segments.iter() {
            if segment.ident == "super" || segment.ident == "crate" || segment.ident == "$crate" {
                self.push_error(
                    String::from("Paths with super / crate / $crate not supported"),
                    path.span(),
                );
            }
        }
        // disallow global paths to any other crates than machine_check and std
        if path.leading_colon.is_some() {
            let crate_segment = path
                .segments
                .first()
                .expect("Global path should have at least one segment");
            let crate_ident = &crate_segment.ident;
            if crate_ident != "machine_check" && crate_ident != "std" {
                self.push_error(
                    String::from(
                        "Only global paths starting with 'machine_check' and 'std' supported",
                    ),
                    path.span(),
                );
            }
        }

        // delegate
        visit::visit_path(self, path);
    }
}
