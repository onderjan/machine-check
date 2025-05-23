use crate::support::rules::{Rule, RuleSegment, Rules};

pub fn abstract_rules() -> Rules {
    let normal_rules = vec![
        Rule {
            has_leading_colon: true,
            segments: vec![
                RuleSegment::Match(String::from("mck")),
                RuleSegment::Match(String::from("abstr")),
                RuleSegment::EndWildcard,
            ],
        },
        Rule {
            has_leading_colon: true,
            segments: vec![
                RuleSegment::Match(String::from("mck")),
                RuleSegment::Match(String::from("forward")),
                RuleSegment::EndWildcard,
            ],
        },
        Rule {
            has_leading_colon: false,
            segments: vec![RuleSegment::ConvertPrefix(
                String::from("__mck_"),
                String::from("__mck_abstr_"),
            )],
        },
        Rule {
            has_leading_colon: false,
            segments: vec![RuleSegment::ConvertPrefix(
                String::from(""),
                String::from("__mck_abstr_"),
            )],
        },
        Rule {
            has_leading_colon: true,
            segments: vec![
                RuleSegment::Match(String::from("mck")),
                RuleSegment::Match(String::from("attr")),
                RuleSegment::EndWildcard,
            ],
        },
        Rule {
            has_leading_colon: true,
            segments: vec![
                RuleSegment::Match(String::from("std")),
                RuleSegment::Match(String::from("clone")),
                RuleSegment::Match(String::from("Clone")),
                RuleSegment::Match(String::from("clone")),
            ],
        },
        Rule {
            has_leading_colon: true,
            segments: vec![
                RuleSegment::Match(String::from("mck")),
                RuleSegment::Match(String::from("refin")),
                RuleSegment::Match(String::from("Refine")),
                RuleSegment::Match(String::from("clean")),
            ],
        },
        Rule {
            has_leading_colon: false,
            segments: vec![
                RuleSegment::Insert(String::from("super")),
                RuleSegment::Match(String::from("Self")),
                RuleSegment::EndWildcard,
            ],
        },
        Rule {
            has_leading_colon: false,
            segments: vec![
                RuleSegment::Insert(String::from("super")),
                RuleSegment::EndWildcard,
            ],
        },
    ];
    let type_rules = vec![
        Rule {
            has_leading_colon: false,
            segments: vec![
                RuleSegment::Insert(String::from("super")),
                RuleSegment::Wildcard,
            ],
        },
        Rule {
            has_leading_colon: true,
            segments: vec![RuleSegment::EndWildcard],
        },
    ];
    Rules::new(normal_rules, type_rules)
}
