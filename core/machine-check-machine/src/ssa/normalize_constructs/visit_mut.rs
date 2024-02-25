use syn::{
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Block, Expr, ExprInfer, Item, Member, Meta, Pat,
};

use crate::util::extract_path_ident;

impl VisitMut for super::Visitor {
    fn visit_item_mut(&mut self, item: &mut syn::Item) {
        match item {
            Item::Struct(_) | Item::Impl(_) => {}
            _ => self.push_error(String::from("Item type not supported"), item.span()),
        }

        // delegate
        visit_mut::visit_item_mut(self, item)
    }

    fn visit_item_struct_mut(&mut self, item_struct: &mut syn::ItemStruct) {
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
            if !is_permitted {
                self.push_error(
                    String::from("Only derive and allow attributes supported on structs"),
                    attr.span(),
                );
            }
        }

        // visit other fields
        self.visit_visibility_mut(&mut item_struct.vis);
        self.visit_ident_mut(&mut item_struct.ident);
        self.visit_generics_mut(&mut item_struct.generics);
        self.visit_fields_mut(&mut item_struct.fields);
    }

    fn visit_generics_mut(&mut self, generics: &mut syn::Generics) {
        if generics.lt_token.is_some() || generics.where_clause.is_some() {
            self.push_error(String::from("Generics not supported"), generics.span());
        }

        // delegate
        visit_mut::visit_generics_mut(self, generics);
    }

    fn visit_item_impl_mut(&mut self, item_impl: &mut syn::ItemImpl) {
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
        visit_mut::visit_item_impl_mut(self, item_impl);
    }

    fn visit_impl_item_mut(&mut self, impl_item: &mut syn::ImplItem) {
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
        visit_mut::visit_impl_item_mut(self, impl_item);
    }

    fn visit_impl_item_fn_mut(&mut self, impl_item_fn: &mut syn::ImplItemFn) {
        if impl_item_fn.defaultness.is_some() {
            self.push_error(
                String::from("Defaultness not supported"),
                impl_item_fn.span(),
            );
        }

        // delegate
        visit_mut::visit_impl_item_fn_mut(self, impl_item_fn);
    }

    fn visit_signature_mut(&mut self, signature: &mut syn::Signature) {
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

        // delegate
        visit_mut::visit_signature_mut(self, signature);
    }

    fn visit_receiver_mut(&mut self, receiver: &mut syn::Receiver) {
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
        visit_mut::visit_receiver_mut(self, receiver);
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // delegate first to avoid spurious path errors
        visit_mut::visit_expr_mut(self, expr);

        let mut taken_expr = Expr::Infer(ExprInfer {
            attrs: vec![],
            underscore_token: Default::default(),
        });
        std::mem::swap(expr, &mut taken_expr);

        let mut processed_expr = match taken_expr {
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
            | Expr::If(_) => {
                // permitted, no conversion
                taken_expr
            }
            Expr::Unary(expr_unary) => {
                // convert unary to call expression
                // conversion cannot be done in unary expression visitor
                self.normalize_unary(expr_unary)
            }
            Expr::Binary(expr_binary) => {
                // convert binary to call expression
                // conversion cannot be done in binary expression visitor
                self.normalize_binary(expr_binary)
            }
            _ => {
                self.push_error(String::from("Expression type not supported"), expr.span());
                taken_expr
            }
        };
        std::mem::swap(expr, &mut processed_expr);
    }

    fn visit_expr_call_mut(&mut self, expr_call: &mut syn::ExprCall) {
        if !matches!(*expr_call.func, Expr::Path(_)) {
            self.push_error(
                String::from("Non-path call functions not supported"),
                expr_call.span(),
            );
        }

        // delegate
        visit_mut::visit_expr_call_mut(self, expr_call);
    }

    fn visit_expr_struct_mut(&mut self, expr_struct: &mut syn::ExprStruct) {
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
        visit_mut::visit_expr_struct_mut(self, expr_struct);
    }

    fn visit_expr_path_mut(&mut self, expr_path: &mut syn::ExprPath) {
        if expr_path.qself.is_some() {
            self.push_error(
                String::from("Qualified self on path not supported"),
                expr_path.span(),
            );
        }

        // delegate
        visit_mut::visit_expr_path_mut(self, expr_path);
    }

    fn visit_expr_if_mut(&mut self, expr_if: &mut syn::ExprIf) {
        // delegate first so path error is not triggered by processing
        visit_mut::visit_expr_if_mut(self, expr_if);

        // process if
        self.process_expr_if(expr_if);
    }

    fn visit_block_mut(&mut self, block: &mut Block) {
        // delegate first
        visit_mut::visit_block_mut(self, block);

        // process block
        self.process_block(block);
    }

    fn visit_pat_mut(&mut self, pat: &mut Pat) {
        println!("Visiting pattern: {}, {:?}", quote::quote!(#pat), pat);
        match pat {
            Pat::Ident(_) | Pat::Lit(_) | Pat::Type(_) => {}
            _ => self.push_error(String::from("Pattern type not supported"), pat.span()),
        };

        // delegate
        visit_mut::visit_pat_mut(self, pat);
    }

    fn visit_pat_ident_mut(&mut self, pat_ident: &mut syn::PatIdent) {
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
        visit_mut::visit_pat_ident_mut(self, pat_ident);
    }

    fn visit_pat_type_mut(&mut self, pat_type: &mut syn::PatType) {
        if !matches!(*pat_type.pat, Pat::Ident(_)) {
            self.push_error(
                String::from("Non-identifier typed pattern not supported"),
                pat_type.span(),
            );
        }

        // delegate
        visit_mut::visit_pat_type_mut(self, pat_type);
    }

    fn visit_type_mut(&mut self, ty: &mut syn::Type) {
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
        visit_mut::visit_type_mut(self, ty);
    }

    fn visit_member_mut(&mut self, member: &mut syn::Member) {
        if !matches!(member, Member::Named(_)) {
            self.push_error(
                String::from("Only named members are supported"),
                member.span(),
            );
        }

        // delegate
        visit_mut::visit_member_mut(self, member);
    }

    fn visit_attribute_mut(&mut self, attribute: &mut syn::Attribute) {
        let mut is_permitted = false;
        if let Meta::List(meta_list) = &attribute.meta {
            if let Some(ident) = extract_path_ident(&meta_list.path) {
                if ident == "allow" {
                    is_permitted = true;
                }
            }
        }
        if !is_permitted {
            self.push_error(
                String::from("Only allow attribute supported in this context"),
                attribute.span(),
            );
        }

        // delegate
        visit_mut::visit_attribute_mut(self, attribute);
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
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
        visit_mut::visit_path_mut(self, path);
    }
}
