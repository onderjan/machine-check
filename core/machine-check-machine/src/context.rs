mod inference;
mod low;
mod name;
mod outer;
mod typed;

pub use inference::WInferenceContext;
pub use low::WLowContext;
pub use outer::WOuterContext;
pub use typed::WTypedContext;

use indexmap::IndexMap;
use syn::Item;

use machine_check_common::PropertyMacros;

use crate::{
    context::name::{expand_property, WNameContext},
    wir::{WIdent, WProperty, WTypeId},
    Errors,
};

pub fn context_from_syn(items: Vec<Item>) -> Result<WLowContext, Errors> {
    let ctx = WNameContext::new(items);
    let ctx = ctx.resolve()?;
    let ctx = ctx.build(&IndexMap::new())?;
    let ctx = ctx.infer()?;
    ctx.lower()
}

pub fn create_property<D>(
    ctx: WOuterContext,
    expr: syn::Expr,
    globals: &IndexMap<WIdent, WTypeId>,
    property_macros: &PropertyMacros<D>,
) -> Result<WProperty, crate::Errors> {
    let property = expand_property(expr, property_macros)?;
    ctx.property_from_expr(globals, property)
}
