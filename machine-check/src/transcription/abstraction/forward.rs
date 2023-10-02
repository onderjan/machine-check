use syn::{visit_mut::VisitMut, ItemStruct};

use crate::transcription::util::{
    generate_derive_attribute,
    path_rule::{self, PathRule, PathRuleSegment},
};

use quote::quote;

pub fn apply(machine: &mut syn::File) -> Result<(), anyhow::Error> {
    // apply transcription to types using path rule transcriptor
    path_rule::apply(machine, path_rules())?;

    // add Default derivation to structs as abstract structs are default unknown
    struct Visitor();
    impl VisitMut for Visitor {
        fn visit_item_struct_mut(&mut self, s: &mut ItemStruct) {
            s.attrs.push(generate_derive_attribute(quote!(Default)));
        }
    }
    Visitor().visit_file_mut(machine);
    Ok(())
}

fn path_rules() -> Vec<PathRule> {
    vec![PathRule {
        has_leading_colon: true,
        segments: vec![
            PathRuleSegment::Ident(String::from("mck")),
            PathRuleSegment::Convert(
                String::from("MachineBitvector"),
                String::from("ThreeValuedBitvector"),
            ),
        ],
    }]
}
