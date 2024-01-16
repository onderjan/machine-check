use syn::{visit_mut::VisitMut, ItemStruct};

use crate::Machine;

use super::util::generate_derive_attribute;

use quote::quote;

use super::{
    support::path_rules::{PathRule, PathRuleSegment, PathRules},
    MachineError,
};

pub(crate) fn apply(machine: &mut Machine) -> Result<(), MachineError> {
    // apply transcription to types using path rule transcriptor
    path_rules().apply_to_items(&mut machine.items)?;

    // add default derive attributes to the structs
    // that easily allow us to make unknown inputs/states
    struct Visitor();
    impl VisitMut for Visitor {
        fn visit_item_struct_mut(&mut self, s: &mut ItemStruct) {
            s.attrs
                .push(generate_derive_attribute(quote!(::std::default::Default)));
        }
    }
    for item in machine.items.iter_mut() {
        Visitor().visit_item_mut(item);
    }
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
