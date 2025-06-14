mod item_impl;
mod item_struct;

use syn::{GenericArgument, Item, Path};

use crate::{
    support::manipulate::{self},
    util::{create_angle_bracketed_path_arguments, create_type_path},
    wir::{
        IntoSyn, WDescription, WElementaryType, WExpr, WExprCall, WGeneralType, WIdent,
        WItemImplTrait, WPanicResult, WPanicResultType, WPath, WSsaLocal, WStmt, WType, YConverted,
        YStage, ZAssignTypes, ZIfPolarity,
    },
};

use self::{
    item_impl::{preprocess_item_impl, process_item_impl},
    item_struct::process_item_struct,
};

#[derive(Clone, Debug, Hash)]
pub struct YAbstr;

#[derive(Clone, Debug, Hash)]
pub struct ZAbstrIfPolarity(bool);

impl IntoSyn<Path> for ZAbstrIfPolarity {
    fn into_syn(self) -> Path {
        if self.0 {
            syn_path::path!(::mck::forward::Test::can_be_true)
        } else {
            syn_path::path!(::mck::forward::Test::can_be_false)
        }
    }
}

impl ZIfPolarity for ZAbstrIfPolarity {}

#[derive(Clone, Debug, Hash)]
pub struct ZAbstr;

impl ZAssignTypes for ZAbstr {
    type Stmt = WStmt<ZAbstr>;
    type FundamentalType = WElementaryType;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprCall>;
    type IfPolarity = ZAbstrIfPolarity;
}

impl YStage for YAbstr {
    type AssignTypes = ZAbstr;
    type InputType = WType<WElementaryType>;
    type OutputType = WPanicResultType<WElementaryType>;
    type FnResult = WPanicResult;
    type Local = WSsaLocal<WGeneralType<WElementaryType>>;
    type ItemImplTrait = WAbstrItemImplTrait;
}

#[derive(Clone, Debug, Hash)]
pub struct WAbstrItemImplTrait {
    pub machine_type: WPath,
    pub trait_: WItemImplTrait,
}

impl IntoSyn<Path> for WAbstrItemImplTrait {
    fn into_syn(self) -> Path {
        let mut trait_path = self.trait_.into_syn();
        trait_path.segments.last_mut().unwrap().arguments = create_angle_bracketed_path_arguments(
            false,
            vec![GenericArgument::Type(create_type_path(
                self.machine_type.clone().into(),
            ))],
            self.machine_type.span(),
        );
        trait_path
    }
}

pub(crate) fn create_abstract_description(
    description: WDescription<YConverted>,
) -> (WDescription<YAbstr>, Vec<Item>) {
    let mut machine_types = Vec::new();
    for item_impl in description.impls.iter() {
        if let Some(ty) = preprocess_item_impl(item_impl) {
            machine_types.push(ty);
        }
    }

    let mut misc_items = Vec::new();

    let mut abstract_description = WDescription::<YAbstr> {
        structs: Vec::new(),
        impls: Vec::new(),
    };

    for item_struct in description.structs {
        let (item_struct, other_impls) = process_item_struct(item_struct);
        abstract_description.structs.push(item_struct);
        misc_items.extend(other_impls.into_iter().map(Item::Impl));
    }

    for item_impl in description.impls {
        let item_impls = process_item_impl(item_impl, &machine_types);
        abstract_description.impls.extend(item_impls);
    }

    // add field-manipulate
    misc_items.extend(
        manipulate::for_abstract_description(&abstract_description)
            .into_iter()
            .map(Item::Impl),
    );

    (abstract_description, misc_items)
}
