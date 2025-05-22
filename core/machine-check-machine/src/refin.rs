use std::collections::HashMap;

use syn::{Item, Type};

use crate::{
    abstr::{WAbstrItemImplTrait, YAbstr, ZAbstrIfPolarity},
    support::manipulate::{self, ManipulateKind},
    wir::{
        IntoSyn, WDescription, WElementaryType, WExpr, WExprCall, WGeneralType, WIdent,
        WItemImplTrait, WPanicResult, WPanicResultType, WSsaLocal, WStmt, YStage, ZAssignTypes,
    },
    BackwardError, Description,
};

use super::support::special_trait::SpecialTrait;

mod item_impl;
mod item_struct;
mod rules;
mod util;

pub(crate) fn create_refinement_description(
    abstract_description: &WDescription<YAbstr>,
) -> Result<Description, BackwardError> {
    // create items to add to the module
    let mut result_structs = Vec::new();
    let mut result_impls = Vec::new();
    let mut ident_special_traits = HashMap::new();

    // first pass
    for item_struct in &abstract_description.structs {
        // apply path rules and push struct
        let item_struct = item_struct::fold_item_struct(item_struct.clone());
        let refin_struct = item_struct.clone().into_syn();
        result_structs.push(refin_struct);
    }

    for item_impl in &abstract_description.impls {
        // look for special traits
        let special_trait = match &item_impl.trait_ {
            Some(trait_) => match &trait_.trait_ {
                WItemImplTrait::Machine => Some(SpecialTrait::Machine),
                WItemImplTrait::Input => Some(SpecialTrait::Input),
                WItemImplTrait::State => Some(SpecialTrait::State),
                WItemImplTrait::Path(_) => None,
            },
            None => None,
        };

        if let Some(special_trait) = special_trait {
            // TODO: make self types be idents for now as no nested modules are supported
            if let Some(ident) = item_impl.self_ty.get_ident() {
                ident_special_traits
                    .entry(ident)
                    .or_insert(Vec::new())
                    .push(special_trait);
            }
        };

        // apply conversion
        let item_impl = item_impl.clone().into_syn();
        result_impls.push(item_impl::fold_item_impl(item_impl)?);
    }

    // second pass, add special impls for special traits
    for item_struct in &abstract_description.structs {
        let special_traits = ident_special_traits
            .remove(&item_struct.ident)
            .unwrap_or(Vec::new());
        for special_trait in special_traits {
            let item_struct = item_struct.clone().into_syn();
            let special_impls = item_struct::special_impls(special_trait, &item_struct)?;
            result_impls.extend(special_impls);
        }
    }

    let mut result_items = Vec::new();

    result_items.extend(result_structs.into_iter().map(Item::Struct));
    result_items.extend(result_impls.into_iter().map(Item::Impl));

    // add field manipulate
    result_items.extend(
        manipulate::for_items(&result_items, ManipulateKind::Backward)
            .into_iter()
            .map(Item::Impl),
    );

    let refinement_machine = Description {
        items: result_items,
    };

    Ok(refinement_machine)
}

#[derive(Clone, Debug, Hash)]
pub struct ZRefin;

impl ZAssignTypes for ZRefin {
    type Stmt = WStmt<ZRefin>;
    type FundamentalType = WBackwardType;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprCall>;
    type IfPolarity = ZAbstrIfPolarity;
}

#[derive(Clone, Debug, Hash)]
pub struct YRefin;

impl YStage for YRefin {
    type AssignTypes = ZRefin;
    type OutputType = WPanicResultType<WElementaryType>;
    type FnResult = WPanicResult;
    type Local = WSsaLocal<WGeneralType<WElementaryType>>;
    type ItemImplTrait = WAbstrItemImplTrait;
}

#[derive(Clone, Debug, Hash)]
pub struct WBackwardType(WElementaryType);

impl IntoSyn<Type> for WBackwardType {
    fn into_syn(self) -> Type {
        self.0.into_syn_type_flavour("backward")
    }
}
