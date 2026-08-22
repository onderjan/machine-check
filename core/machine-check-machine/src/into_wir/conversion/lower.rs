/*use indexmap::IndexMap;

use crate::{
    context::{WInferredContext, WLowContext},
    into_wir::Errors,
    util::ident_creator::IdentCreator,
    wir::{
        WDescription, WExpr, WExprLowCall, WFnSignature, WIdent, WImplItemType, WItemFn,
        WItemFnBody, WItemImpl, WItemStruct, WMckNew, WPartialPath, WPartialSegment, WProperty,
        WStmt, WStmtAssign, WSubproperty, WSubpropertyFunc, WTacLocal, WTypeId, YLowered, YTac,
    },
};
use mck::{concr::ConcreteBitvector, misc::RBound};


pub fn lower_description(
    mut ctx: WInferredContext,
    description: WDescription<YTac>,
) -> Result<(WLowContext, WDescription<YLowered>), Errors> {
    let mut structs = Vec::new();
    let mut impls = Vec::new();
    for item_struct in description.structs {
        structs.push(lower_item_struct(&mut ctx, item_struct));
    }
    let structs = Errors::flat_result(structs)?;

    for item_impl in description.impls {
        impls.push(lower_item_impl(&mut ctx, item_impl));
    }

    let impls = Errors::flat_result(impls)?;

    let ctx = ctx.lower()?;

    Ok((ctx, WDescription { structs, impls }))
}

pub fn lower_property(
    mut ctx: WInferredContext,
    property: WProperty<YTac>,
) -> Result<(WLowContext, WProperty<YLowered>), Errors> {
    let mut subproperties = Vec::new();

    for subproperty in property.subproperties {
        let subproperty = match subproperty {
            WSubproperty::Func(subproperty_func) => WSubproperty::Func(WSubpropertyFunc {
                parent: subproperty_func.parent,
                func: lower_item_fn(&mut ctx, /*None,*/ subproperty_func.func)?,
                children: subproperty_func.children,
                display: subproperty_func.display,
            }),
            WSubproperty::FixedPoint(fixed_point) => WSubproperty::FixedPoint(fixed_point),
            WSubproperty::Next(next) => WSubproperty::Next(next),
        };

        subproperties.push(subproperty);
    }

    let ctx = ctx.lower()?;

    Ok((ctx, WProperty { subproperties }))
}
*/
