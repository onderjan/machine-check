use syn::visit_mut::VisitMut;
use syn::{Ident, Path};

use anyhow::anyhow;

#[derive(Clone)]
pub enum PathRuleSegment {
    Ident(String),
    Convert(String, String),
    Wildcard,
}

#[derive(Clone)]
pub struct PathRule {
    pub has_leading_colon: bool,
    pub segments: Vec<PathRuleSegment>,
}

pub fn apply(file: &mut syn::File, rules: Vec<PathRule>) -> Result<(), anyhow::Error> {
    let mut visitor = Visitor::new(rules);
    visitor.visit_file_mut(file);
    visitor.first_error.map_or(Ok(()), Err)
}

pub fn apply_to_item_struct(
    s: &mut syn::ItemStruct,
    rules: Vec<PathRule>,
) -> Result<(), anyhow::Error> {
    let mut visitor = Visitor::new(rules);
    visitor.visit_item_struct_mut(s);
    visitor.first_error.map_or(Ok(()), Err)
}

pub fn apply_to_path(p: &mut syn::Path, rules: Vec<PathRule>) -> Result<(), anyhow::Error> {
    let mut visitor = Visitor::new(rules);
    visitor.apply_to_path(p)?;
    visitor.first_error.map_or(Ok(()), Err)
}

struct Visitor {
    first_error: Option<anyhow::Error>,
    rules: Vec<PathRule>,
}

impl VisitMut for Visitor {
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

impl Visitor {
    fn new(rules: Vec<PathRule>) -> Self {
        Self {
            first_error: None,
            rules,
        }
    }

    fn apply_to_path(&mut self, path: &mut Path) -> Result<(), anyhow::Error> {
        // use the first rule that applies
        for rule in &mut self.rules {
            if match_rule(path, rule) {
                return Ok(());
            }
        }
        Err(anyhow!("no rule matches path {:?}", path))
    }
}

fn match_rule(path: &mut Path, rule: &PathRule) -> bool {
    // only match rule if leading colon requirement matches
    if rule.has_leading_colon != path.leading_colon.is_some() {
        return false;
    }
    let mut segments = path.segments.iter();
    for rule_segment in &rule.segments {
        let Some(segment) = segments.next() else {
            // the path is too short for the rule
            return false;
        };
        match rule_segment {
            PathRuleSegment::Ident(match_ident_string)
            | PathRuleSegment::Convert(match_ident_string, _) => {
                // only match if the path ident is the same
                let ident_string = segment.ident.to_string();

                if ident_string.as_str() != match_ident_string.as_str() {
                    return false;
                }
            }
            PathRuleSegment::Wildcard => {
                // always match
            }
        }
    }
    // all segments matched, replace appropriate segment identifier strings and break
    let mut segments = path.segments.iter_mut();
    for rule_segment in &rule.segments {
        let Some(segment) = segments.next() else {
            // should never happen
            return false;
        };
        if let PathRuleSegment::Convert(_, replace_ident_string) = rule_segment {
            // replace ident
            segment.ident = Ident::new(replace_ident_string.as_str(), segment.ident.span());
        }
    }
    true
}
