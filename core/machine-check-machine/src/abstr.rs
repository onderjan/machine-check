/*mod item_impl;
mod item_struct;*/

use syn::{GenericArgument, Item, Path, Type};

use crate::{
    util::{create_angle_bracketed_path_arguments, create_type_path},
    wir::{
        IntoSyn, WDescription, WExpr, WExprLowCall, WIdent, WItemImplTrait, WPath, WSsaLocal,
        WStmt, WTypeId, YSsa, YStage, ZAssignTypes, ZIfPolarity,
    },
};

/*
use self::{
    item_impl::{preprocess_item_impl, process_item_impl},
    item_struct::process_item_struct,
};*/

#[derive(Clone, Debug, Hash)]
pub struct YAbstr;

#[derive(Clone, Debug, Hash)]
pub struct ZAbstrIfPolarity(pub bool);

impl IntoSyn<Path> for ZAbstrIfPolarity {
    fn into_syn(self, _type_fn: &impl Fn(WTypeId) -> Type) -> Path {
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
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprLowCall>;
    type IfPolarity = ZAbstrIfPolarity;
}

impl YStage for YAbstr {
    type AssignTypes = ZAbstr;
    type FnResult = WIdent;
    type Local = WSsaLocal;
    type ItemImplTrait = WAbstrItemImplTrait;
}

#[derive(Clone, Debug, Hash)]
pub struct WAbstrItemImplTrait {
    pub machine_type: WPath,
    pub trait_: WItemImplTrait,
}

impl IntoSyn<Path> for WAbstrItemImplTrait {
    fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Path {
        let mut trait_path = self.trait_.into_syn(type_fn);
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
    description: WDescription<YSsa>,
) -> (WDescription<YAbstr>, Vec<Item>) {
    todo!("Create abstract description");
    /*
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

    (abstract_description, misc_items)
    */
}
