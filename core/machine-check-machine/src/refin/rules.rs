use crate::support::path_rules::{PathRule, PathRuleSegment, PathRules};

pub fn refinement_normal() -> PathRules {
    PathRules::new(vec![
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Match(String::from("mck")),
                PathRuleSegment::Convert(String::from("abstr"), String::from("refin")),
                PathRuleSegment::EndWildcard,
            ],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Match(String::from("mck")),
                PathRuleSegment::Convert(String::from("forward"), String::from("backward")),
                PathRuleSegment::EndWildcard,
            ],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Match(String::from("mck")),
                PathRuleSegment::Match(String::from("misc")),
                PathRuleSegment::EndWildcard,
            ],
        },
        PathRule {
            has_leading_colon: false,
            segments: vec![PathRuleSegment::ConvertPrefix(
                String::from("__mck_"),
                String::from("__mck_refin_"),
            )],
        },
        PathRule {
            has_leading_colon: false,
            segments: vec![PathRuleSegment::ConvertPrefix(
                String::from(""),
                String::from("__mck_refin_"),
            )],
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
    ])
}

pub fn refinement_type() -> PathRules {
    PathRules::new(vec![
        PathRule {
            has_leading_colon: false,
            segments: vec![PathRuleSegment::Wildcard],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Match(String::from("mck")),
                PathRuleSegment::Convert(String::from("abstr"), String::from("refin")),
                PathRuleSegment::EndWildcard,
            ],
        },
    ])
}

pub fn abstract_normal() -> PathRules {
    PathRules::new(vec![
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Match(String::from("mck")),
                PathRuleSegment::Match(String::from("abstr")),
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
            has_leading_colon: false,
            segments: vec![PathRuleSegment::ConvertPrefix(
                String::from("__mck_"),
                String::from("__mck_abstr_"),
            )],
        },
        PathRule {
            has_leading_colon: false,
            segments: vec![PathRuleSegment::ConvertPrefix(
                String::from(""),
                String::from("__mck_abstr_"),
            )],
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
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Match(String::from("std")),
                PathRuleSegment::Match(String::from("default")),
                PathRuleSegment::Match(String::from("Default")),
                PathRuleSegment::Match(String::from("default")),
            ],
        },
    ])
}

pub fn abstract_type() -> PathRules {
    PathRules::new(vec![
        PathRule {
            has_leading_colon: false,
            segments: vec![
                PathRuleSegment::Insert(String::from("super")),
                PathRuleSegment::Wildcard,
            ],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![PathRuleSegment::EndWildcard],
        },
    ])
}

pub fn clone_normal() -> PathRules {
    PathRules::new(vec![
        PathRule {
            has_leading_colon: false,
            segments: vec![PathRuleSegment::ConvertPrefix(
                String::from("__mck_"),
                String::from("__mck_clone_"),
            )],
        },
        PathRule {
            has_leading_colon: false,
            segments: vec![PathRuleSegment::ConvertPrefix(
                String::from(""),
                String::from("clone_"),
            )],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![PathRuleSegment::EndWildcard],
        },
    ])
}

pub fn clone_type() -> PathRules {
    PathRules::new(vec![
        PathRule {
            has_leading_colon: false,
            segments: vec![PathRuleSegment::EndWildcard],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![PathRuleSegment::EndWildcard],
        },
    ])
}
