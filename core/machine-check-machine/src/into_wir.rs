mod conversion;
mod property;

use indexmap::IndexMap;
use machine_check_common::PropertyMacros;
use syn::Item;

use crate::{
    context::{WLowContext, WOuterContext},
    into_wir::conversion::context_from_syn,
    wir::{WIdent, WProperty, WTypeId},
};

pub fn create_context(items: Vec<Item>) -> Result<WLowContext, crate::Errors> {
    context_from_syn(items)
}

pub fn create_property<D>(
    ctx: WOuterContext,
    expr: syn::Expr,
    globals: &IndexMap<WIdent, WTypeId>,
    property_macros: &PropertyMacros<D>,
) -> Result<WProperty, crate::Errors> {
    property::create_from_syn(ctx, expr, globals, property_macros)
}
