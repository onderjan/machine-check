use syn::{ImplItem, Item, ItemImpl, Path, PathArguments, Type};
use syn_path::path;

use crate::machine::{
    support::struct_rules::StructRules,
    util::{create_ident, create_impl_item_type},
};

mod args;
mod item_impl_fn;
mod local_visitor;

use anyhow::anyhow;

use super::{rules, SpecialTrait};

pub(super) fn apply(
    refinement_items: &mut Vec<Item>,
    item_impl: &ItemImpl,
) -> Result<Option<SpecialTrait>, anyhow::Error> {
    let self_ty = item_impl.self_ty.as_ref();

    let Type::Path(self_ty) = self_ty else {
        return Err(anyhow!("Non-path impl type not supported"));
    };

    let Some(self_ty_ident) = self_ty.path.get_ident() else {
        return Err(anyhow!("Non-ident impl type not supported"));
    };

    let converter = ImplConverter {
        abstract_rules: StructRules::new(
            self_ty_ident.clone(),
            rules::abstract_normal(),
            rules::abstract_type(),
        ),
        refinement_rules: StructRules::new(
            self_ty_ident.clone(),
            rules::refinement_normal(),
            rules::refinement_type(),
        ),
    };

    let (converted_item_impl, special_trait) = converter.convert(item_impl.clone())?;
    refinement_items.push(Item::Impl(converted_item_impl));
    Ok(special_trait)
}

pub struct ImplConverter {
    pub abstract_rules: StructRules,
    pub refinement_rules: StructRules,
}

impl ImplConverter {
    fn convert(
        &self,
        item_impl: ItemImpl,
    ) -> Result<(ItemImpl, Option<SpecialTrait>), anyhow::Error> {
        let mut items = Vec::<ImplItem>::new();
        for item in &item_impl.items {
            match item {
                ImplItem::Fn(item_fn) => {
                    items.push(ImplItem::Fn(self.transcribe_impl_item_fn(item_fn)?))
                }
                ImplItem::Type(item_type) => {
                    // just clone to preserve pointed-to type, now in refinement module context
                    items.push(ImplItem::Type(item_type.clone()));
                }
                _ => return Err(anyhow!("Impl item type {:?} not supported", item)),
            }
        }

        let mut item_impl = item_impl;
        item_impl.items = items;

        let mut special_trait = None;
        if let Some((None, trait_path, _)) = &item_impl.trait_ {
            special_trait = if is_abstr_input_trait(trait_path) {
                Some(SpecialTrait::Input)
            } else if is_abstr_state_trait(trait_path) {
                Some(SpecialTrait::State)
            } else if is_abstr_machine_trait(trait_path) {
                Some(SpecialTrait::Machine)
            } else {
                None
            };

            if special_trait.is_some() {
                // add abstract type
                let type_ident = create_ident("Abstract");
                let type_assign = self
                    .abstract_rules
                    .convert_type((*item_impl.self_ty).clone())?;
                item_impl.items.push(ImplItem::Type(create_impl_item_type(
                    type_ident,
                    type_assign,
                )));
            }
        }

        if let Some(trait_) = &mut item_impl.trait_ {
            trait_.1 = self
                .refinement_rules
                .convert_normal_path(trait_.1.clone())?;
        }

        Ok((item_impl, special_trait))
    }
}

fn is_abstr_input_trait(trait_path: &Path) -> bool {
    trait_path == &path!(::mck::abstr::Input)
}

fn is_abstr_state_trait(trait_path: &Path) -> bool {
    trait_path == &path!(::mck::abstr::State)
}

fn is_abstr_machine_trait(trait_path: &Path) -> bool {
    // strip generics
    let mut trait_path = trait_path.clone();
    for seg in &mut trait_path.segments {
        seg.arguments = PathArguments::None;
    }

    trait_path == path!(::mck::abstr::Machine)
}
