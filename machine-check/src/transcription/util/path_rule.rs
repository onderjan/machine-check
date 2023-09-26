use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{Ident, Path};

use crate::transcription::util::generate_derive_attribute;

#[allow(dead_code)]
pub enum PathRuleSegment {
    Ident(String),
    Convert(String, String),
    Wildcard,
}

pub struct PathRule {
    pub has_leading_colon: bool,
    pub segments: Vec<PathRuleSegment>,
}

pub fn transcribe(machine: &mut syn::File, rules: Vec<PathRule>) -> Result<(), anyhow::Error> {
    let mut visitor = Visitor {
        first_error: None,
        rules,
    };
    visitor.visit_file_mut(machine);
    visitor.first_error.map_or(Ok(()), Err)
}

struct Visitor {
    first_error: Option<anyhow::Error>,
    rules: Vec<PathRule>,
}

impl VisitMut for Visitor {
    fn visit_item_struct_mut(&mut self, i: &mut syn::ItemStruct) {
        // add Default derivation as abstract structs are default unknown
        i.attrs.push(generate_derive_attribute(quote!(Default)));
        // delegate
        syn::visit_mut::visit_item_struct_mut(self, i);
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if let Err(err) = self.transcribe_path(path) {
            if self.first_error.is_none() {
                self.first_error = Some(err);
            }
        }
        // delegate
        syn::visit_mut::visit_path_mut(self, path);
    }
}

impl Visitor {
    fn transcribe_path(&mut self, path: &mut Path) -> Result<(), anyhow::Error> {
        // use the first rule that applies
        println!("Matching path {:?}", path);
        'rule_loop: for rule in &self.rules {
            // only match rule if leading colon requirement matches
            if rule.has_leading_colon != path.leading_colon.is_some() {
                continue;
            }
            println!("Matched leading colon");
            let mut segments = path.segments.iter();
            for rule_segment in &rule.segments {
                let Some(segment) = segments.next() else {
                    // the path is too short for the rule
                    println!("Path too short");
                    continue 'rule_loop;
                };
                match rule_segment {
                    PathRuleSegment::Ident(match_ident_string)
                    | PathRuleSegment::Convert(match_ident_string, _) => {
                        // only match if the path ident is the same
                        let ident_string = segment.ident.to_string();

                        if ident_string.as_str() != match_ident_string.as_str() {
                            println!(
                                "Strings not matching: '{}' != '{}'",
                                ident_string, match_ident_string
                            );
                            continue 'rule_loop;
                        }
                    }
                    PathRuleSegment::Wildcard => {
                        // always match
                    }
                }
            }
            println!("Matched {:?}", quote!(#path));
            // all segments matched, transcribe and break
            let mut segments = path.segments.iter_mut();
            for rule_segment in &rule.segments {
                let Some(segment) = segments.next() else {
                    // should never happen
                    continue 'rule_loop;
                };
                if let PathRuleSegment::Convert(_, replace_ident_string) = rule_segment {
                    // replace ident
                    segment.ident = Ident::new(replace_ident_string.as_str(), segment.ident.span());
                }
            }
            break;
        }
        Ok(())
    }
}
