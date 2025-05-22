use syn::{
    spanned::Spanned, GenericArgument, Ident, ImplItem, ItemImpl, Path, PathArguments, Type,
};

use crate::{
    abstr::YAbstr,
    support::rules::Rules,
    util::create_path_segment,
    wir::{IntoSyn, WItemImpl},
    BackwardError, BackwardErrorType,
};

mod args;
mod item_impl_fn;

use super::{rules, WRefinItemImplTrait};

pub fn fold_item_impl(item_impl: WItemImpl<YAbstr>) -> Result<ItemImpl, BackwardError> {
    let previous_result = {
        let item_impl = item_impl.clone().into_syn();
        // convert implementation
        let self_ty_name = extract_self_type_ident(&item_impl)?.to_string();

        let converter = ImplConverter {
            clone_rules: rules::clone_rules().with_self_ty_name(self_ty_name.clone()),
            abstract_rules: rules::abstract_rules().with_self_ty_name(self_ty_name.clone()),
            refinement_rules: rules::refinement_rules().with_self_ty_name(self_ty_name.clone()),
        };
        let mut converted_item_impl = converter.convert(item_impl)?;

        if let Some((_, path, _)) = &mut converted_item_impl.trait_ {
            // convert generics arguments to super so that they still point to the original types
            convert_generic_arguments_to_super(path)?;
        };
        converted_item_impl
    };

    let new_result = {
        let mut impl_item_fns = Vec::new();
        let self_ty = &item_impl.self_ty;
        for impl_item_fn in item_impl.impl_item_fns {
            impl_item_fns.push(item_impl_fn::fold_impl_item_fn(impl_item_fn, self_ty));
        }

        let trait_ = match item_impl.trait_ {
            Some(trait_) => Some(WRefinItemImplTrait {
                machine_type: trait_.machine_type,
                trait_: trait_.trait_,
            }),
            None => None,
        };

        WItemImpl {
            self_ty: item_impl.self_ty,
            trait_,
            impl_item_fns,
            impl_item_types: item_impl.impl_item_types,
        }
    };

    //let result = new_result.into_syn();
    let result = previous_result;

    Ok(result)
}

fn extract_self_type_ident(item_impl: &ItemImpl) -> Result<&Ident, BackwardError> {
    let self_ty = item_impl.self_ty.as_ref();

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
    Ok(self_ty_ident)
}

fn convert_generic_arguments_to_super(path: &mut Path) -> Result<(), BackwardError> {
    for segment in &mut path.segments {
        match &mut segment.arguments {
            PathArguments::None => {
                // do nothing
            }
            PathArguments::AngleBracketed(angle_bracketed) => {
                for argument in &mut angle_bracketed.args {
                    let span = argument.span();
                    let GenericArgument::Type(Type::Path(type_path)) = argument else {
                        return Err(BackwardError::new(
                            BackwardErrorType::UnsupportedConstruct(String::from(
                                "Only path-based type generic arguments are supported",
                            )),
                            span,
                        ));
                    };
                    if type_path.path.leading_colon.is_none() {
                        type_path
                            .path
                            .segments
                            .insert(0, create_path_segment(Ident::new("super", span)));
                    }
                }
            }
            PathArguments::Parenthesized(_) => {
                return Err(BackwardError::new(
                    BackwardErrorType::UnsupportedConstruct(String::from(
                        "Parenthesized generic arguments not supported",
                    )),
                    segment.arguments.span(),
                ));
            }
        }
    }
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
