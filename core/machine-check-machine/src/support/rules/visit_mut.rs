use syn::{punctuated::Punctuated, visit_mut::VisitMut, Expr, Ident, Member};

use crate::util::create_path_from_ident;

impl<'a> VisitMut for super::Visitor<'a> {
    fn visit_item_struct_mut(&mut self, node: &mut syn::ItemStruct) {
        for it in &mut node.attrs {
            self.visit_attribute_mut(it);
        }
        self.visit_visibility_mut(&mut node.vis);
        // treat specially by considering struct ident to be a type
        let prev_inside_type = self.inside_type;
        self.inside_type = true;
        self.visit_ident_mut(&mut node.ident);
        self.inside_type = prev_inside_type;
        self.visit_generics_mut(&mut node.generics);
        self.visit_fields_mut(&mut node.fields);
    }

    fn visit_pat_struct_mut(&mut self, node: &mut syn::PatStruct) {
        for it in &mut node.attrs {
            self.visit_attribute_mut(it);
        }
        if let Some(it) = &mut node.qself {
            self.visit_qself_mut(it);
        }
        // treat specially by considering struct path to be a type
        let prev_inside_type = self.inside_type;
        self.inside_type = true;
        self.visit_path_mut(&mut node.path);
        self.inside_type = prev_inside_type;

        for mut el in Punctuated::pairs_mut(&mut node.fields) {
            let it = el.value_mut();
            self.visit_field_pat_mut(it);
        }
        if let Some(it) = &mut node.rest {
            self.visit_pat_rest_mut(it);
        }
    }

    fn visit_expr_struct_mut(&mut self, node: &mut syn::ExprStruct) {
        for it in &mut node.attrs {
            self.visit_attribute_mut(it);
        }
        if let Some(it) = &mut node.qself {
            self.visit_qself_mut(it);
        }
        // treat specially by considering struct path to be a type
        let prev_inside_type = self.inside_type;
        self.inside_type = true;
        self.visit_path_mut(&mut node.path);
        self.inside_type = prev_inside_type;

        for mut el in node.fields.pairs_mut() {
            let it = el.value_mut();
            // handle shorthands gracefully: add the colon token first to convert from shorthand
            it.colon_token = Some(Default::default());
            self.visit_field_value_mut(it);
            // after visiting the field (and potentially changing the expression path),
            // if it is possible to use shorthand, convert to it
            if let Member::Named(member) = &it.member {
                if let Expr::Path(path) = &it.expr {
                    if path.path.is_ident(member) {
                        it.colon_token = None;
                    }
                }
            }
        }
        if let Some(it) = &mut node.rest {
            self.visit_expr_mut(it);
        }
    }

    fn visit_field_mut(&mut self, node: &mut syn::Field) {
        for it in &mut node.attrs {
            self.visit_attribute_mut(it);
        }
        self.visit_visibility_mut(&mut node.vis);
        self.visit_field_mutability_mut(&mut node.mutability);
        // treat specially by not going into field
        self.visit_type_mut(&mut node.ty);
    }

    fn visit_member_mut(&mut self, _: &mut Member) {
        // do not go into the member
    }

    fn visit_attribute_mut(&mut self, _: &mut syn::Attribute) {
        // do not visit attribute paths
    }

    fn visit_type_mut(&mut self, ty: &mut syn::Type) {
        let prev_inside_type = self.inside_type;
        self.inside_type = true;
        syn::visit_mut::visit_type_mut(self, ty);
        self.inside_type = prev_inside_type;
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if let Err(err) = self.apply_to_path(path) {
            if self.first_error.is_none() {
                self.first_error = Some(err);
            }
        }
        // delegate
        let prev_inside_path = self.inside_path;
        self.inside_path = true;
        syn::visit_mut::visit_path_mut(self, path);
        self.inside_path = prev_inside_path;
    }

    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        if self.inside_path {
            return;
        }
        let mut path = create_path_from_ident(ident.clone());
        if let Err(err) = self.apply_to_path(&mut path) {
            if self.first_error.is_none() {
                self.first_error = Some(err);
            }
        }
        let result_ident = path
            .get_ident()
            .expect("Identifier should be converted to identifier")
            .clone();
        *ident = result_ident;
    }
}
