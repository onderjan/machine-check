mod impl_item_fn;
mod item_struct;

use syn::{ImplItem, Item};

use crate::{support::field_manipulate, MachineDescription};

use self::{impl_item_fn::process_impl_item_fn, item_struct::process_impl_item_struct};

use super::{
    support::path_rules::{PathRule, PathRuleSegment, PathRules},
    MachineError,
};

pub(crate) fn create_abstract_machine(
    ssa_machine: &MachineDescription,
) -> Result<MachineDescription, MachineError> {
    // expecting the concrete machine in SSA form
    let mut abstract_machine = ssa_machine.clone();
    // apply transcription to types using path rule transcriptor
    path_rules().apply_to_items(&mut abstract_machine.items)?;

    let mut processed_items = Vec::new();

    for item in abstract_machine.items.drain(..) {
        match item {
            syn::Item::Impl(mut item_impl) => {
                for impl_item in item_impl.items.iter_mut() {
                    if let ImplItem::Fn(impl_item_fn) = impl_item {
                        process_impl_item_fn(impl_item_fn)?;
                    }
                }
                processed_items.push(Item::Impl(item_impl));
            }
            syn::Item::Struct(item_struct) => {
                processed_items.extend(process_impl_item_struct(item_struct)?);
            }
            _ => panic!("Unexpected item type"),
        }
    }
    abstract_machine.items = processed_items;

    // add field-manipulate to items
    field_manipulate::apply_to_items(&mut abstract_machine.items, "abstr")?;

    Ok(abstract_machine)
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
