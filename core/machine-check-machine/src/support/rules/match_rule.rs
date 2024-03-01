use syn::Path;

use crate::util::{create_ident, create_path_segment};

use super::{Rule, RuleSegment};

pub fn match_rule(path: &mut Path, rule: &Rule) -> bool {
    // only match rule if leading colon requirement matches
    if rule.has_leading_colon != path.leading_colon.is_some() {
        return false;
    }
    let mut segments = path.segments.iter();
    for rule_segment in &rule.segments {
        if matches!(rule_segment, RuleSegment::EndWildcard) {
            // exhaust segments
            for _ in segments.by_ref() {}
            continue;
        }

        if matches!(rule_segment, RuleSegment::Insert(_)) {
            // skip this rule segment for matching
            continue;
        }

        let Some(segment) = segments.next() else {
            // the path is too short for the rule
            return false;
        };
        match rule_segment {
            RuleSegment::Match(match_ident_string)
            | RuleSegment::Convert(match_ident_string, _) => {
                // only match if the path ident is the same
                let ident_string = segment.ident.to_string();

                if ident_string.as_str() != match_ident_string.as_str() {
                    return false;
                }
            }
            RuleSegment::ConvertPrefix(match_prefix, _) => {
                // only match if the ident starts with prefix
                let ident_string = segment.ident.to_string();

                if !ident_string.as_str().starts_with(match_prefix) {
                    return false;
                }
            }
            RuleSegment::Wildcard | RuleSegment::Insert(_) | RuleSegment::EndWildcard => {
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
        if let RuleSegment::Insert(insert_name) = rule_segment {
            // insert without increasing previous segments counter
            path.segments
                .push(create_path_segment(create_ident(insert_name)));
            continue;
        }
        let mut segment = previous_segments[i].clone();
        match rule_segment {
            RuleSegment::Convert(_, replacement_name) => {
                // replace ident
                segment.ident = create_ident(replacement_name);
            }
            RuleSegment::ConvertPrefix(match_prefix, replacement_prefix) => {
                let segment_name = segment.ident.to_string();
                let stripped_name = segment_name.strip_prefix(match_prefix).unwrap();
                let replacement_name = replacement_prefix.to_string() + stripped_name;
                segment.ident = create_ident(&replacement_name);
            }
            RuleSegment::EndWildcard => {
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
