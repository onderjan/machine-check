use std::collections::HashMap;

use indexmap::IndexMap;
use syn::Item;

use crate::{
    context::{WContextBuilder, WLowContext},
    into_wir::{
        conversion::{expand_macros, resolve_use},
        Errors,
    },
};

pub fn context_from_syn(mut items: Vec<Item>) -> Result<WLowContext, Errors> {
    let mut use_map = HashMap::new();
    loop {
        use_map.extend(resolve_use::extract_use_map(&mut items)?);
        resolve_use::resolve_use_items(&mut items, &use_map)?;
        if !expand_macros::expand_in_items(&mut items)? {
            break;
        }
    }
    resolve_use::remove_use(&mut items)?;

    let mut builder = WContextBuilder::new();
    builder.add_syn_items(items)?;
    builder.build(&IndexMap::new())?.infer()?.lower()
}
