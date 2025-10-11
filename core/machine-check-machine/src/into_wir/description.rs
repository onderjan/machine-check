use std::collections::HashMap;

use syn::Item;

use crate::{
    into_wir::{
        conversion::{
            convert_indexing, convert_to_ssa, convert_total, convert_types, expand_macros,
            infer_types, resolve_use,
        },
        from_syn, Error, Errors,
    },
    wir::{WDescription, YConverted, YTac},
};

pub fn description_from_syn(
    mut items: Vec<Item>,
) -> Result<(WDescription<YConverted>, Vec<String>), Errors> {
    let mut use_map = HashMap::new();
    loop {
        use_map.extend(resolve_use::extract_use_map(&mut items)?);
        resolve_use::resolve_use_items(&mut items, &use_map)?;
        if !expand_macros::expand_in_items(&mut items)? {
            break;
        }
    }

    resolve_use::remove_use(&mut items)?;

    let w_description = tac_from_items(items.into_iter())?;
    let w_description = convert_indexing::convert_description(w_description);
    let (w_description, panic_messages) = convert_total::convert_description(w_description);
    let w_description = convert_to_ssa::convert_description(w_description)?;
    let w_description = infer_types::infer_description(w_description)?;
    let w_description = convert_types::convert_description(w_description)?;

    Ok((w_description, panic_messages))
}

fn tac_from_items(item_iter: impl Iterator<Item = Item>) -> Result<WDescription<YTac>, Errors> {
    let mut structs = Vec::new();
    let mut impls = Vec::new();
    let mut errors = Vec::new();
    for item in item_iter {
        match item {
            Item::Struct(item) => structs.push(from_syn::fold_item_struct(item)),
            Item::Impl(item) => impls.push(from_syn::fold_item_impl(item)),
            _ => errors.push(Error::unsupported_syn_construct("Item kind", &item)),
        }
    }

    let structs = Errors::flat_result(structs);
    let impls = Errors::flat_result(impls);
    let (structs, impls) = Errors::combine_and_vec(structs, impls, errors)?;

    Ok(WDescription { structs, impls })
}
