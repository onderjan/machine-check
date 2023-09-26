use std::collections::HashMap;

use syn::{visit_mut::VisitMut, Ident, PatType, Path};

use super::mark_type_path::convert_type_path_to_original;

pub struct IdentVisitor {
    pub rules: HashMap<String, String>,
    pub prefix_rule: Option<(String, String)>,
}

impl IdentVisitor {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            prefix_rule: None,
        }
    }

    pub fn transcribe_ident(&self, ident: &mut Ident) {
        if let Some(replacement_string) = self.rules.get(&ident.to_string()) {
            *ident = Ident::new(replacement_string, ident.span());
        }

        if let Some(prefix_rule) = &self.prefix_rule {
            let ident_string = ident.to_string();
            let rule_stripped = ident_string
                .strip_prefix(&prefix_rule.0)
                .unwrap_or(&ident_string);
            let replacement_string = format!("{}{}", prefix_rule.1, rule_stripped);
            *ident = Ident::new(replacement_string.as_str(), ident.span());
        }
    }

    pub fn transcribe_path(&self, path: &mut Path) {
        // only transcribe idents, those do not start with leading colon and have exactly one segment
        if path.leading_colon.is_some() {
            return;
        }
        let mut segments_mut = path.segments.iter_mut();
        let Some(ident_segment) = segments_mut.next() else {
            return;
        };
        if segments_mut.next().is_some() {
            return;
        };

        let ident = &mut ident_segment.ident;
        self.transcribe_ident(ident);
    }
}

impl VisitMut for IdentVisitor {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        self.transcribe_path(path);
        // do not delegate to idents
        //syn::visit_mut::visit_path_mut(self, path);
    }

    fn visit_ident_mut(&mut self, i: &mut Ident) {
        self.transcribe_ident(i);
        // delegate
        syn::visit_mut::visit_ident_mut(self, i);
    }

    fn visit_expr_field_mut(&mut self, i: &mut syn::ExprField) {
        self.visit_expr_mut(&mut i.base);
    }

    fn visit_expr_struct_mut(&mut self, i: &mut syn::ExprStruct) {
        i.path = convert_type_path_to_original(&i.path);
        // do not delegate to path
        for field in &mut i.fields {
            self.visit_expr_mut(&mut field.expr);
        }
    }

    fn visit_pat_type_mut(&mut self, _: &mut PatType) {
        // do not delegate
    }
}
