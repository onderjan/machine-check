use std::rc::Rc;

use proc_macro2::{Ident, Span};

use syn::spanned::Spanned;
use syn::Path;
use syn::{visit_mut::VisitMut, Item};

use crate::util::{create_path_from_ident, extract_path_ident};

mod match_rule;
mod visit_mut;

#[derive(thiserror::Error, Clone, Debug)]
#[error("No rule matched")]
pub struct NoRuleMatch(pub Span);

#[derive(Clone, Debug)]
pub enum RuleSegment {
    Match(String),
    Convert(String, String),
    ConvertPrefix(String, String),
    Insert(String),
    Wildcard,
    EndWildcard,
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub has_leading_colon: bool,
    pub segments: Vec<RuleSegment>,
}

#[derive(Clone)]
pub struct Rules {
    normal_rules: Rc<Vec<Rule>>,
    type_rules: Rc<Vec<Rule>>,
    self_ty_name: Option<String>,
}

impl Rules {
    pub fn new(normal_rules: Vec<Rule>, type_rules: Vec<Rule>) -> Rules {
        Rules {
            normal_rules: Rc::new(normal_rules),
            type_rules: Rc::new(type_rules),
            self_ty_name: None,
        }
    }

    pub fn with_self_ty_name(&self, self_ty_name: String) -> Rules {
        Rules {
            normal_rules: Rc::clone(&self.normal_rules),
            type_rules: Rc::clone(&self.type_rules),
            self_ty_name: Some(self_ty_name),
        }
    }

    pub(crate) fn apply_to_items(&self, items: &mut [Item]) -> Result<(), NoRuleMatch> {
        let mut visitor = Visitor::new(self);
        for item in items.iter_mut() {
            visitor.visit_item_mut(item);
        }
        visitor.first_error.map_or(Ok(()), Err)
    }

    pub(crate) fn apply_to_item_struct(&self, s: &mut syn::ItemStruct) -> Result<(), NoRuleMatch> {
        let mut visitor = Visitor::new(self);
        visitor.visit_item_struct_mut(s);
        visitor.first_error.map_or(Ok(()), Err)
    }

    pub(crate) fn apply_to_stmt(&self, s: &mut syn::Stmt) -> Result<(), NoRuleMatch> {
        let mut visitor = Visitor::new(self);
        visitor.visit_stmt_mut(s);
        visitor.first_error.map_or(Ok(()), Err)
    }

    pub(crate) fn apply_to_expr(&self, s: &mut syn::Expr) -> Result<(), NoRuleMatch> {
        let mut visitor = Visitor::new(self);
        visitor.visit_expr_mut(s);
        visitor.first_error.map_or(Ok(()), Err)
    }

    pub(crate) fn apply_to_expr_struct(&self, s: &mut syn::ExprStruct) -> Result<(), NoRuleMatch> {
        let mut visitor = Visitor::new(self);
        visitor.visit_expr_struct_mut(s);
        visitor.first_error.map_or(Ok(()), Err)
    }

    pub(crate) fn convert_normal_ident(
        &self,
        ident: syn::Ident,
    ) -> Result<syn::Ident, NoRuleMatch> {
        Ok(
            extract_path_ident(&self.convert_normal_path(create_path_from_ident(ident))?)
                .expect("Ident should not be converted to a non-ident path")
                .clone(),
        )
    }

    pub(crate) fn convert_normal_path(
        &self,
        mut path: syn::Path,
    ) -> Result<syn::Path, NoRuleMatch> {
        let mut visitor = Visitor::new(self);
        visitor.apply_to_path(&mut path)?;
        visitor.first_error.map_or(Ok(path), Err)
    }

    pub(crate) fn convert_type_path(&self, mut path: syn::Path) -> Result<syn::Path, NoRuleMatch> {
        let mut visitor = Visitor::new(self);
        visitor.inside_type = true;
        visitor.apply_to_path(&mut path)?;
        visitor.first_error.map_or(Ok(path), Err)
    }

    pub(crate) fn convert_type(&self, mut ty: syn::Type) -> Result<syn::Type, NoRuleMatch> {
        let mut visitor = Visitor::new(self);
        visitor.visit_type_mut(&mut ty);
        visitor.first_error.map_or(Ok(ty), Err)
    }
}

struct Visitor<'a> {
    first_error: Option<NoRuleMatch>,
    rules: &'a Rules,
    inside_type: bool,
    inside_path: bool,
}

impl<'a> Visitor<'a> {
    fn new(rules: &'a Rules) -> Self {
        Self {
            first_error: None,
            rules,
            inside_type: false,
            inside_path: false,
        }
    }

    fn apply_to_path(&mut self, path: &mut Path) -> Result<(), NoRuleMatch> {
        let used_rules = if self.inside_type {
            self.rules.type_rules.as_ref()
        } else {
            self.rules.normal_rules.as_ref()
        };
        let mut matched_rule = false;
        // use the first rule that applies
        for rule in used_rules {
            if match_rule::match_rule(path, rule) {
                matched_rule = true;
                break;
            }
        }

        if matched_rule {
            if let Some(self_ty_name) = &self.rules.self_ty_name {
                // replace Self by type name after rule matching
                for path_segment in path.segments.iter_mut() {
                    if path_segment.ident == "Self" {
                        path_segment.ident = Ident::new(self_ty_name, path_segment.span());
                    }
                }
            }

            return Ok(());
        }
        Err(NoRuleMatch(path.span()))
    }
}
