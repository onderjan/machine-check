use crate::machine::util::path_rules::{PathRule, PathRuleSegment, PathRules};

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
    ])
}

pub fn refinement_type() -> PathRules {
    PathRules::new(vec![PathRule {
        has_leading_colon: false,
        segments: vec![PathRuleSegment::Wildcard],
    }])
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
    ])
}

pub fn abstract_type() -> PathRules {
    PathRules::new(vec![PathRule {
        has_leading_colon: false,
        segments: vec![
            PathRuleSegment::Insert(String::from("super")),
            PathRuleSegment::Wildcard,
        ],
    }])
}
