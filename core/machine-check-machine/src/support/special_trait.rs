use syn::{punctuated::Punctuated, ItemImpl, Path, PathArguments};

use crate::util::{create_ident, create_path_segment};

pub enum SpecialTrait {
    Machine,
    Input,
    State,
}

pub fn special_trait_impl(item_impl: &ItemImpl, flavour: &str) -> Option<SpecialTrait> {
    let Some((None, trait_path, _)) = &item_impl.trait_ else {
        return None;
    };
    special_trait_path(trait_path, flavour)
}

pub fn special_trait_path(trait_path: &Path, flavour: &str) -> Option<SpecialTrait> {
    if is_abstr_input_trait(trait_path, flavour) {
        Some(SpecialTrait::Input)
    } else if is_abstr_state_trait(trait_path, flavour) {
        Some(SpecialTrait::State)
    } else if is_abstr_machine_trait(trait_path, flavour) {
        Some(SpecialTrait::Machine)
    } else {
        None
    }
}

fn create_trait_path(flavour: &str, ty: &str) -> Path {
    Path {
        leading_colon: Some(Default::default()),
        segments: Punctuated::from_iter(
            vec![
                create_path_segment(create_ident("mck")),
                create_path_segment(create_ident(flavour)),
                create_path_segment(create_ident(ty)),
            ]
            .into_iter(),
        ),
    }
}

fn is_abstr_input_trait(trait_path: &Path, flavour: &str) -> bool {
    trait_path == &create_trait_path(flavour, "Input")
}

fn is_abstr_state_trait(trait_path: &Path, flavour: &str) -> bool {
    trait_path == &create_trait_path(flavour, "State")
}

fn is_abstr_machine_trait(trait_path: &Path, flavour: &str) -> bool {
    // strip generics
    let mut trait_path = trait_path.clone();
    for seg in &mut trait_path.segments {
        seg.arguments = PathArguments::None;
    }

    trait_path == create_trait_path(flavour, "Machine")
}
