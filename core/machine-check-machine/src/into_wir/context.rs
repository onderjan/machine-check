use std::collections::HashMap;

use indexmap::IndexMap;
use syn::Item;

use crate::{
    context::{WContextBuilder, WLowContext},
    into_wir::{
        conversion::{expand_macros, resolve_use},
        from_syn::{self},
        Error, Errors,
    },
    wir::{WIdent, WTotalPath},
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
    let mut errors = Vec::new();

    for item in items {
        match item {
            Item::Struct(item) => {
                let path = WTotalPath::from_ident(WIdent::from_syn_ident(item.ident.clone()))
                    .without_generics();
                match from_syn::fold_item_struct(&mut builder, item) {
                    Ok(item_struct) => {
                        builder.add_struct(path, item_struct);
                    }
                    Err(err) => errors.push(err),
                }
            }
            Item::Impl(item) => match from_syn::fold_item_impl(&mut builder, item) {
                Ok(item_impl) => {
                    builder.add_impl(item_impl)?;
                }
                Err(err) => errors.push(err),
            },
            _ => errors.push(Error::unsupported_syn_construct("Item kind", &item).into()),
        }
    }

    Errors::errors_vec_to_result(errors)?;

    builder.build(&IndexMap::new())?.infer()?.lower()
}
