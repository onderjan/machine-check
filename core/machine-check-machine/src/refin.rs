use std::collections::HashMap;

use syn::{Ident, Item, Type};

use crate::{
    abstr::{WAbstrItemImplTrait, YAbstr, ZAbstrIfPolarity},
    support::manipulate::{self, ManipulateKind},
    wir::{
        IntoSyn, WDescription, WElementaryType, WExpr, WExprCall, WGeneralType, WIdent,
        WPanicResult, WPanicResultType, WSsaLocal, WStmt, YStage, ZAssignTypes,
    },
    BackwardError, Description,
};

use super::support::special_trait::{special_trait_impl, SpecialTrait};

mod item_impl;
mod item_struct;
mod rules;
mod util;

pub(crate) fn create_refinement_description(
    abstract_description: &WDescription<YAbstr>,
) -> Result<Description, BackwardError> {
    // create items to add to the module
    let mut result_items = Vec::<Item>::new();
    let mut ident_special_traits = HashMap::<Ident, SpecialTrait>::new();

    // first pass
    for item_struct in &abstract_description.structs {
        // apply path rules and push struct
        let item_struct = item_struct::fold_item_struct(item_struct.clone());

        let refin_struct = item_struct.clone().into_syn();
        //rules::refinement_rules().apply_to_item_struct(&mut refin_struct)?;
        result_items.push(Item::Struct(refin_struct));
    }

    for item_impl in &abstract_description.impls {
        let item_impl = item_impl.clone().into_syn();

        // apply conversion
        item_impl::apply(&mut result_items, &item_impl)?;
        // look for special traits
        if let Some(special_trait) = special_trait_impl(&item_impl, "forward") {
            if let Type::Path(ty) = item_impl.self_ty.as_ref() {
                if let Some(ident) = ty.path.get_ident() {
                    ident_special_traits.insert(ident.clone(), special_trait);
                }
            }
        };
    }

    // second pass, add special impls for special traits
    for item_struct in &abstract_description.structs {
        if let Some(special_trait) = ident_special_traits.remove(&item_struct.ident.to_syn_ident())
        {
            let item_struct = item_struct.clone().into_syn();
            item_struct::add_special_impls(special_trait, &mut result_items, &item_struct)?;
        }
    }

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
