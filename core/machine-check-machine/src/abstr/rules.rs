use crate::support::path_rules::{PathRule, PathRuleSegment, PathRules};

pub fn path_rules() -> PathRules {
    PathRules::new(vec![
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Match(String::from("mck")),
                PathRuleSegment::Convert(String::from("concr"), String::from("abstr")),
                PathRuleSegment::EndWildcard,
            ],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Match(String::from("mck")),
                PathRuleSegment::Match(String::from("forward")),
                PathRuleSegment::EndWildcard,
            ],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Match(String::from("mck")),
                PathRuleSegment::Match(String::from("attr")),
                PathRuleSegment::EndWildcard,
            ],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Match(String::from("std")),
                PathRuleSegment::Match(String::from("clone")),
                PathRuleSegment::Match(String::from("Clone")),
                PathRuleSegment::Match(String::from("clone")),
            ],
        },
        PathRule {
            has_leading_colon: false,
            segments: vec![PathRuleSegment::Wildcard],
        },
        PathRule {
            has_leading_colon: false,
            segments: vec![
                PathRuleSegment::Match(String::from("Self")),
                PathRuleSegment::EndWildcard,
            ],
        },
    ])
}
