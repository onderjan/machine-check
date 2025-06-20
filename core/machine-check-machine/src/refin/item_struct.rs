use syn::ItemImpl;

use crate::{
    support::meta_eq::meta_eq_impl,
    wir::{WElementaryType, WField, WIdent, WItemStruct, WPath, WPathSegment},
};

use self::{meta::meta_impl, refine::refine_impl};

use super::{SpecialTrait, WBackwardElementaryType};

mod meta;
mod refine;

pub fn fold_item_struct(
    item_struct: WItemStruct<WElementaryType>,
) -> WItemStruct<WBackwardElementaryType> {
    let fields = item_struct
        .fields
        .into_iter()
        .map(|field| WField {
            visibility: field.visibility,
            ident: field.ident,
            ty: WBackwardElementaryType(field.ty),
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
    item_struct: &WItemStruct<WElementaryType>,
) -> Vec<ItemImpl> {
    let abstr_type_path = WPath {
        leading_colon: None,
        segments: vec![
            WPathSegment {
                ident: WIdent::new(String::from("super"), item_struct.ident.span()),
            },
            WPathSegment {
                ident: item_struct.ident.clone(),
            },
        ],
    };

    match special_trait {
        SpecialTrait::Input | SpecialTrait::State => {
            // add Meta and Refine implementations
            vec![
                meta_impl(item_struct, &abstr_type_path),
                refine_impl(item_struct, &abstr_type_path),
                meta_eq_impl(item_struct),
            ]
        }

        SpecialTrait::Machine => {
            // add Refine implementation
            vec![refine_impl(item_struct, &abstr_type_path)]
        }
    }
}
