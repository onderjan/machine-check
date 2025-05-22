use syn::{ItemImpl, ItemStruct};

use crate::{
    support::meta_eq::meta_eq_impl,
    util::create_path_from_ident,
    wir::{WElementaryType, WField, WItemStruct},
    BackwardError,
};

use self::{meta::meta_impl, refine::refine_impl};

use super::{rules, SpecialTrait, WBackwardType};

mod meta;
mod refine;

pub fn fold_item_struct(item_struct: WItemStruct<WElementaryType>) -> WItemStruct<WBackwardType> {
    let fields = item_struct
        .fields
        .into_iter()
        .map(|field| WField {
            ident: field.ident,
            ty: WBackwardType(field.ty),
        })
        .collect();

    WItemStruct {
        visibility: item_struct.visibility,
        derives: item_struct.derives,
        ident: item_struct.ident,
        fields,
    }
}

pub(super) fn special_impls(
    special_trait: SpecialTrait,
    item_struct: &ItemStruct,
) -> Result<Vec<ItemImpl>, BackwardError> {
    let abstr_type_path = rules::abstract_rules()
        .convert_type_path(create_path_from_ident(item_struct.ident.clone()))?;

    Ok(match special_trait {
        SpecialTrait::Input | SpecialTrait::State => {
            // add Meta and Refine implementations
            vec![
                meta_impl(item_struct, &abstr_type_path)?,
                refine_impl(item_struct, &abstr_type_path)?,
                meta_eq_impl(item_struct),
            ]
        }

        SpecialTrait::Machine => {
            // add Refine implementation
            vec![refine_impl(item_struct, &abstr_type_path)?]
        }
    })
}
