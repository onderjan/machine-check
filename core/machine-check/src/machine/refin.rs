use proc_macro2::Span;
use syn::{Ident, Item};

use crate::machine::util::path_rule::{PathRule, PathRuleSegment};

use super::util::create_item_mod;

mod item_impl;
mod item_struct;

pub fn apply(abstract_machine_file: &mut syn::File) -> anyhow::Result<()> {
    // the refinement machine will be in a new module at the end of the file

    // create items to add to the module
    let mut mark_file_items = Vec::<Item>::new();
    for item in &abstract_machine_file.items {
        match item {
            Item::Struct(s) => item_struct::apply(&mut mark_file_items, s)?,
            Item::Impl(i) => item_impl::apply(&mut mark_file_items, i)?,
            _ => {
                return Err(anyhow::anyhow!("Item type {:?} not supported", item));
            }
        };
    }
    // create new module at the end of the file that will contain the refinement
    let mod_mark = Item::Mod(create_item_mod(
        syn::Visibility::Public(Default::default()),
        Ident::new("refin", Span::call_site()),
        mark_file_items,
    ));
    abstract_machine_file.items.push(mod_mark);
    Ok(())
}

pub fn mark_path_normal_rules() -> Vec<PathRule> {
    vec![
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
    ]
}

pub fn mark_path_type_rules() -> Vec<PathRule> {
    vec![PathRule {
        has_leading_colon: false,
        segments: vec![PathRuleSegment::Wildcard],
    }]
}

pub fn abstract_path_normal_rules() -> Vec<PathRule> {
    vec![
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
    ]
}

pub fn abstract_path_type_rules() -> Vec<PathRule> {
    vec![PathRule {
        has_leading_colon: false,
        segments: vec![
            PathRuleSegment::Insert(String::from("super")),
            PathRuleSegment::Wildcard,
        ],
    }]
}
