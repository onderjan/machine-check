use syn::{
    spanned::Spanned, GenericArgument, Ident, ImplItem, Item, ItemImpl, PathArguments, Type,
};

use crate::{support::rules::Rules, util::create_path_segment, BackwardError, BackwardErrorType};

mod args;
mod item_impl_fn;

use super::rules;

pub(super) fn apply(
    refinement_items: &mut Vec<Item>,
    item_impl: &ItemImpl,
) -> Result<(), BackwardError> {
    let self_ty = item_impl.self_ty.as_ref();
    println!("Applying refinement to impl {}", quote::quote!(#self_ty));

    let Type::Path(self_ty) = self_ty else {
        return Err(BackwardError::new(
            BackwardErrorType::UnsupportedConstruct(String::from(
                "Non-path impl type not supported",
            )),
            self_ty.span(),
        ));
    };

    let Some(self_ty_ident) = self_ty.path.get_ident() else {
        return Err(BackwardError::new(
            BackwardErrorType::UnsupportedConstruct(String::from(
                "Non-ident impl type not supported",
            )),
            self_ty.span(),
        ));
    };

    let self_ty_name = self_ty_ident.to_string();

    let converter = ImplConverter {
        clone_rules: rules::clone_rules().with_self_ty_name(self_ty_name.clone()),
        abstract_rules: rules::abstract_rules().with_self_ty_name(self_ty_name.clone()),
        refinement_rules: rules::refinement_rules().with_self_ty_name(self_ty_name.clone()),
    };
    let mut converted_item_impl = converter.convert(item_impl.clone())?;

    if let Some((_, path, _)) = &mut converted_item_impl.trait_ {
        // convert generic parameters
        for segment in &mut path.segments {
            if let PathArguments::AngleBracketed(angle_bracketed) = &mut segment.arguments {
                for argument in &mut angle_bracketed.args {
                    let span = argument.span();
                    if let GenericArgument::Type(Type::Path(type_path)) = argument {
                        if type_path.path.leading_colon.is_none() {
                            type_path
                                .path
                                .segments
                                .insert(0, create_path_segment(Ident::new("super", span)));
                        }
                    }
                }
            }
        }
    };

    refinement_items.push(Item::Impl(converted_item_impl));

    Ok(())
}

pub struct ImplConverter {
    pub clone_rules: Rules,
    pub abstract_rules: Rules,
    pub refinement_rules: Rules,
}

impl ImplConverter {
    fn convert(&self, item_impl: ItemImpl) -> Result<ItemImpl, BackwardError> {
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
                _ => panic!("Unexpected impl item type"),
            }
        }

        let mut item_impl = item_impl;
        item_impl.items = items;

        if let Some(trait_) = &mut item_impl.trait_ {
            trait_.1 = self.refinement_rules.convert_type_path(trait_.1.clone())?;
        }

        Ok(item_impl)
    }
}
