use crate::support::rules::{Rule, RuleSegment, Rules};

pub fn path_rules() -> Rules {
    let normal_rules = vec![
        Rule {
            has_leading_colon: true,
            segments: vec![
                RuleSegment::Match(String::from("mck")),
                RuleSegment::Convert(String::from("concr"), String::from("abstr")),
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
            has_leading_colon: false,
            segments: vec![RuleSegment::Wildcard],
        },
        Rule {
            has_leading_colon: false,
            segments: vec![
                RuleSegment::Match(String::from("Self")),
                RuleSegment::EndWildcard,
            ],
        },
    ];
    Rules::new(normal_rules.clone(), normal_rules)
}
