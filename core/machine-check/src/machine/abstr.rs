use syn::{visit_mut::VisitMut, ItemStruct};

use crate::machine::util::{
    generate_derive_attribute,
    path_rules::{PathRule, PathRuleSegment},
};

use quote::quote;

use super::util::path_rules::PathRules;

pub fn apply(machine: &mut syn::File) -> Result<(), anyhow::Error> {
    // apply transcription to types using path rule transcriptor
    path_rules().apply_to_file(machine)?;

    // add default derive attributes to the structs
    // that easily allow us to make unknown inputs/states
    struct Visitor();
    impl VisitMut for Visitor {
        fn visit_item_struct_mut(&mut self, s: &mut ItemStruct) {
            s.attrs
                .push(generate_derive_attribute(quote!(::std::default::Default)));
        }
    }
    Visitor().visit_file_mut(machine);
    Ok(())
}

fn path_rules() -> PathRules {
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
            has_leading_colon: false,
            segments: vec![PathRuleSegment::Wildcard],
        },
    ])
}
