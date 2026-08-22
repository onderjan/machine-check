use std::collections::HashMap;

use syn::{Item, Type, TypePath};

use crate::{
    context::{WInferenceContext, WInferredContext, WLowContext},
    into_wir::{
        conversion::{convert_to_ssa, expand_macros, lower, resolve_use},
        from_syn::{self, fold_partial_path},
        Error, Errors,
    },
    wir::{WDescription, WIdent, WPath, YSsa, YTac},
};

pub fn description_from_syn(
    mut items: Vec<Item>,
) -> Result<(WLowContext, WDescription<YSsa>, Vec<String>), Errors> {
    let mut use_map = HashMap::new();
    loop {
        use_map.extend(resolve_use::extract_use_map(&mut items)?);
        resolve_use::resolve_use_items(&mut items, &use_map)?;
        if !expand_macros::expand_in_items(&mut items)? {
            break;
        }
    }

    resolve_use::remove_use(&mut items)?;

    let (ctx, description) = tac_from_items(items)?;
    //let w_description = convert_indexing::convert_description(w_description);
    /*let (w_description, panic_messages) =
    convert_total::convert_description(&mut ctx, w_description);*/
    let panic_messages = Vec::new();
    let (mut ctx, description) = lower::lower_description(ctx, description)?;
    let description = convert_to_ssa::convert_description(&mut ctx, description)?;
    Ok((ctx, description, panic_messages))
}

fn tac_from_items(item_iter: Vec<Item>) -> Result<(WInferredContext, WDescription<YTac>), Errors> {
    let mut ctx = WInferenceContext::new();

    let mut structs = Vec::new();
    let mut impls = Vec::new();
    let mut errors = Vec::new();

    // TODO: rewrite to 1. signatures, 2. bodies
    // first iteration: structs
    for item in &item_iter {
        if let Item::Struct(item) = item {
            let path =
                WPath::from_ident(WIdent::from_syn_ident(item.ident.clone())).without_generics();
            let struct_def = from_syn::fold_item_struct(&mut ctx, item.clone());
            if let Ok(struct_def) = &struct_def {
                ctx.add_struct_sig(path, struct_def.clone());
            }
            structs.push(struct_def);
        }
    }

    // second iteration: impls
    for item in item_iter {
        match item {
            Item::Struct(_item) => {}
            Item::Impl(item) => {
                let Type::Path(TypePath {
                    qself: None,
                    path: self_path,
                }) = *item.self_ty.clone()
                else {
                    todo!("Non-path impl type");
                };

                let path = fold_partial_path(self_path)?.without_generics();

                let impl_def = from_syn::fold_item_impl(&mut ctx, item);
                if let Ok(impl_def) = &impl_def {
                    ctx.add_impl_sig(path, impl_def.clone());
                }
                impls.push(impl_def)
            }
            _ => errors.push(Error::unsupported_syn_construct("Item kind", &item)),
        }
    }
    let structs = Errors::flat_result(structs);
    let impls = Errors::flat_result(impls);
    let (structs, impls) = Errors::combine_and_vec(structs, impls, errors)?;

    let ctx = ctx.infer_impls(impls.as_slice())?;
    let description = WDescription { structs, impls };

    Ok((ctx, description))
}
