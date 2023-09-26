use crate::transcription::util::path_rule::{self, PathRule, PathRuleSegment};

pub fn transcribe(machine: &mut syn::File) -> Result<(), anyhow::Error> {
    // transcribe types using path rule transcriptor
    let rules = vec![
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Ident(String::from("mck")),
                PathRuleSegment::Convert(
                    String::from("MachineArray"),
                    String::from("ThreeValuedArray"),
                ),
            ],
        },
        PathRule {
            has_leading_colon: true,
            segments: vec![
                PathRuleSegment::Ident(String::from("mck")),
                PathRuleSegment::Convert(
                    String::from("MachineBitvector"),
                    String::from("ThreeValuedBitvector"),
                ),
            ],
        },
    ];
    path_rule::transcribe(machine, rules)
}
