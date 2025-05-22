mod item_impl;
mod item_struct;

use syn::Item;

use crate::{
    support::manipulate::{self, ManipulateKind},
    wir::{
        IntoSyn, WDescription, WElementaryType, WGeneralType, WPanicResult, WPanicResultType,
        WSsaLocal, YConverted, YStage, ZConverted,
    },
    Description,
};

use self::{
    item_impl::{preprocess_item_impl, process_item_impl},
    item_struct::process_item_struct,
};

use super::Error;

pub struct YAbstr;

impl YStage for YAbstr {
    type AssignTypes = ZConverted;
    type OutputType = WPanicResultType<WElementaryType>;
    type FnResult = WPanicResult;
    type Local = WSsaLocal<WGeneralType<WElementaryType>>;
}

pub(crate) fn create_abstract_description(
    description: WDescription<YConverted>,
) -> Result<Description, Error> {
    let mut machine_types = Vec::new();
    for item_impl in description.impls.iter() {
        if let Some(ty) = preprocess_item_impl(item_impl)? {
            machine_types.push(ty);
        }
    }

    let mut processed_items = Vec::new();

    let mut w_description = WDescription::<YAbstr> {
        structs: Vec::new(),
        impls: Vec::new(),
    };

    for item_struct in description.structs {
        let (item_struct, other_impls) = process_item_struct(item_struct)?;
        w_description.structs.push(item_struct);
        processed_items.extend(other_impls.into_iter().map(Item::Impl));
    }

    for item_impl in description.impls {
        let item_impl = item_impl.into_syn();
        let item_impls = process_item_impl(item_impl, &machine_types)?;
        processed_items.extend(item_impls.into_iter().map(Item::Impl));
    }

    let mut abstract_description = Description {
        items: processed_items,
    };

    abstract_description.items.extend(
        w_description
            .structs
            .into_iter()
            .map(|a| Item::Struct(a.into_syn())),
    );

    abstract_description.items.extend(
        w_description
            .impls
            .into_iter()
            .map(|a| Item::Impl(a.into_syn())),
    );

    // add field-manipulate to items
    manipulate::apply_to_items(&mut abstract_description.items, ManipulateKind::Forward);

    Ok(abstract_description)
}
