use std::collections::HashMap;

use syn::{Item, Path, Type, TypePath};

use crate::{
    into_wir::{
        conversion::{convert_to_ssa, convert_total, convert_types, expand_macros, resolve_use},
        from_syn, Error, Errors,
    },
    wir::{WDescription, WInferenceContext, WInferredContext, YLowered, YTac},
};

pub fn description_from_syn(
    mut items: Vec<Item>,
) -> Result<(WInferredContext, WDescription<YLowered>, Vec<String>), Errors> {
    let mut use_map = HashMap::new();
    loop {
        use_map.extend(resolve_use::extract_use_map(&mut items)?);
        resolve_use::resolve_use_items(&mut items, &use_map)?;
        if !expand_macros::expand_in_items(&mut items)? {
            break;
        }
    }

    resolve_use::remove_use(&mut items)?;

    let mut ctx = WInferenceContext::new();
    let w_description = tac_from_items(&mut ctx, items.into_iter())?;
    //let w_description = convert_indexing::convert_description(w_description);
    let mut ctx = ctx.into_total()?;
    let (w_description, panic_messages) =
        convert_total::convert_description(&mut ctx, w_description);
    let w_description = convert_to_ssa::convert_description(&mut ctx, w_description)?;
    let w_description = convert_types::lower_description(&mut ctx, w_description)?;
    //ctx.lower()?;
    Ok((ctx, w_description, panic_messages))
}

fn tac_from_items(
    ctx: &mut WInferenceContext,
    item_iter: impl Iterator<Item = Item>,
) -> Result<WDescription<YTac>, Errors> {
    let mut structs = Vec::new();
    let mut impls = Vec::new();
    let mut errors = Vec::new();

    for item in item_iter {
        match item {
            Item::Struct(item) => {
                let ty = Type::Path(TypePath {
                    qself: None,
                    path: Path::from(item.ident.clone()),
                });
                let struct_def = from_syn::fold_item_struct(ctx, item);
                if let Ok(struct_def) = &struct_def {
                    ctx.add_struct_def(ty, struct_def);
                }
                structs.push(struct_def);
            }
            Item::Impl(item) => impls.push(from_syn::fold_item_impl(ctx, item)),
            _ => errors.push(Error::unsupported_syn_construct("Item kind", &item)),
        }
    }
    let structs = Errors::flat_result(structs);
    let impls = Errors::flat_result(impls);
    let (structs, impls) = Errors::combine_and_vec(structs, impls, errors)?;

    ctx.resolve_types(impls.as_slice())?;

    Ok(WDescription { structs, impls })
}
