use std::rc::Rc;

use proc_macro2::Ident;
use syn::spanned::Spanned;
use syn::Path;
use syn::{visit_mut::VisitMut, Item};

use crate::{
    util::{create_ident, create_path_segment},
    MachineError,
};

#[derive(Clone, Debug)]
pub enum PathRuleSegment {
    Match(String),
    Convert(String, String),
    ConvertPrefix(String, String),
    Insert(String),
    Wildcard,
    EndWildcard,
}

#[derive(Clone, Debug)]
pub struct PathRule {
    pub has_leading_colon: bool,
    pub segments: Vec<PathRuleSegment>,
}

#[derive(Clone)]
pub struct PathRules {
    rules: Rc<Vec<PathRule>>,
    self_ty_name: Option<String>,
}

impl PathRules {
    pub fn new(rules: Vec<PathRule>) -> PathRules {
        PathRules {
            rules: Rc::new(rules),
            self_ty_name: None,
        }
    }

    pub fn with_self_ty_name(&self, self_ty_name: String) -> PathRules {
        PathRules {
            rules: Rc::clone(&self.rules),
            self_ty_name: Some(self_ty_name),
        }
    }

    pub(crate) fn apply_to_items(&self, items: &mut [Item]) -> Result<(), MachineError> {
        let mut visitor = Visitor::new(&self.rules, &self.self_ty_name);
        for item in items.iter_mut() {
            visitor.visit_item_mut(item);
        }
        visitor.first_error.map_or(Ok(()), Err)
    }

    pub(crate) fn apply_to_item_struct(&self, s: &mut syn::ItemStruct) -> Result<(), MachineError> {
        let mut visitor = Visitor::new(&self.rules, &self.self_ty_name);
        visitor.visit_item_struct_mut(s);
        visitor.first_error.map_or(Ok(()), Err)
    }

    pub(crate) fn convert_path(&self, path: syn::Path) -> Result<syn::Path, MachineError> {
        let mut visitor = Visitor::new(&self.rules, &self.self_ty_name);
        let mut path = path;
        visitor.apply_to_path(&mut path)?;
        visitor.first_error.map_or(Ok(path), Err)
    }
}

struct Visitor<'a> {
    first_error: Option<MachineError>,
    rules: &'a Vec<PathRule>,
    self_ty_name: &'a Option<String>,
}

impl<'a> VisitMut for Visitor<'a> {
    fn visit_attribute_mut(&mut self, _: &mut syn::Attribute) {
        // do not visit attribute paths
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if let Err(err) = self.apply_to_path(path) {
            if self.first_error.is_none() {
                self.first_error = Some(err);
            }
        }
        // delegate
        syn::visit_mut::visit_path_mut(self, path);
    }
}

impl<'a> Visitor<'a> {
    fn new(rules: &'a Vec<PathRule>, self_ty_name: &'a Option<String>) -> Self {
        Self {
            first_error: None,
            rules,
            self_ty_name,
        }
    }

    fn apply_to_path(&mut self, path: &mut Path) -> Result<(), MachineError> {
        let mut matched_rule = false;
        // use the first rule that applies
        for rule in self.rules {
            if match_rule(path, rule) {
                matched_rule = true;
                break;
            }
        }

        if matched_rule {
            if let Some(self_ty_name) = self.self_ty_name {
                // replace Self by type name after rule matching
                for path_segment in path.segments.iter_mut() {
                    if path_segment.ident == "Self" {
                        path_segment.ident = Ident::new(self_ty_name, path_segment.span());
                    }
                }
            }
            return Ok(());
        }

        Err(MachineError(format!(
            "no rule matches path {:?}, rules: {:?}",
            path, self.rules
        )))
    }
}

fn match_rule(path: &mut Path, rule: &PathRule) -> bool {
    // only match rule if leading colon requirement matches
    if rule.has_leading_colon != path.leading_colon.is_some() {
        return false;
    }
    let mut segments = path.segments.iter();
    for rule_segment in &rule.segments {
        if matches!(rule_segment, PathRuleSegment::EndWildcard) {
            // exhaust segments
            for _ in segments.by_ref() {}
            continue;
        }

        if matches!(rule_segment, PathRuleSegment::Insert(_)) {
            // skip this rule segment for matching
            continue;
        }

        let Some(segment) = segments.next() else {
            // the path is too short for the rule
            return false;
        };
        match rule_segment {
            PathRuleSegment::Match(match_ident_string)
            | PathRuleSegment::Convert(match_ident_string, _) => {
                // only match if the path ident is the same
                let ident_string = segment.ident.to_string();

                if ident_string.as_str() != match_ident_string.as_str() {
                    return false;
                }
            }
            PathRuleSegment::ConvertPrefix(match_prefix, _) => {
                // only match if the ident starts with prefix
                let ident_string = segment.ident.to_string();

                if !ident_string.as_str().starts_with(match_prefix) {
                    return false;
                }
            }
            PathRuleSegment::Wildcard
            | PathRuleSegment::Insert(_)
            | PathRuleSegment::EndWildcard => {
                // always match
            }
        }
    }
    if segments.next().is_some() {
        // some segment not matched
        return false;
    }

    // all segments matched, replace appropriate segment identifier strings and break
    let previous_segments = path.segments.clone();
    path.segments.clear();
    let mut i = 0;
    for rule_segment in &rule.segments {
        if let PathRuleSegment::Insert(insert_name) = rule_segment {
            // insert without increasing previous segments counter
            path.segments
                .push(create_path_segment(create_ident(insert_name)));
            continue;
        }
        let mut segment = previous_segments[i].clone();
        match rule_segment {
            PathRuleSegment::Convert(_, replacement_name) => {
                // replace ident
                segment.ident = create_ident(replacement_name);
            }
            PathRuleSegment::ConvertPrefix(match_prefix, replacement_prefix) => {
                let segment_name = segment.ident.to_string();
                let stripped_name = segment_name.strip_prefix(match_prefix).unwrap();
                let replacement_name = replacement_prefix.to_string() + stripped_name;
                segment.ident = create_ident(&replacement_name);
            }
            PathRuleSegment::EndWildcard => {
                // add all remaining segments and end
                for j in i..previous_segments.len() {
                    path.segments.push(previous_segments[j].clone());
                }
                return true;
            }
            _ => {}
        }
        path.segments.push(segment);
        i += 1;
    }
    true
}
