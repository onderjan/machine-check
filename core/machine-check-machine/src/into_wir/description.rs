use std::collections::HashMap;

use indexmap::IndexMap;
use syn::{Item, Path, Type, TypePath};

use crate::{
    context::{WInferenceContext, WInferredContext, WLowContext},
    into_wir::{
        conversion::{convert_to_ssa, expand_macros, lower, resolve_use},
        from_syn, Error, Errors,
    },
    wir::{
        WDescription, WImplFnSignature, WImplTypeSignature, WItemImpl, WItemStruct, WPath,
        WPathSegment, WSignature, WSignatures, WStructSignature, YSsa, YTac,
    },
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

    let (ctx, description) = tac_from_items(items.into_iter())?;
    //let w_description = convert_indexing::convert_description(w_description);
    /*let (w_description, panic_messages) =
    convert_total::convert_description(&mut ctx, w_description);*/
    let panic_messages = Vec::new();
    let (mut ctx, description) = lower::lower_description(ctx, description)?;
    let description = convert_to_ssa::convert_description(&mut ctx, description)?;
    Ok((ctx, description, panic_messages))
}

fn tac_from_items(
    item_iter: impl Iterator<Item = Item>,
) -> Result<(WInferredContext, WDescription<YTac>), Errors> {
    let mut ctx = WInferenceContext::new();

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
                let struct_def = from_syn::fold_item_struct(&mut ctx, item);
                if let Ok(struct_def) = &struct_def {
                    ctx.add_struct_def(ty, struct_def);
                }
                structs.push(struct_def);
            }
            Item::Impl(item) => impls.push(from_syn::fold_item_impl(&mut ctx, item)),
            _ => errors.push(Error::unsupported_syn_construct("Item kind", &item)),
        }
    }
    let structs = Errors::flat_result(structs);
    let impls = Errors::flat_result(impls);
    let (structs, impls) = Errors::combine_and_vec(structs, impls, errors)?;

    let signatures = generate_signatures(&structs, &impls);
    let ctx = ctx.infer_impls(signatures, impls.as_slice())?;
    let description = WDescription { structs, impls };

    Ok((ctx, description))
}

pub fn generate_signatures(structs: &[WItemStruct], impls: &[WItemImpl<YTac>]) -> WSignatures {
    let mut signatures = IndexMap::new();
    for item_struct in structs {
        let path = WPath::from_ident(item_struct.ident.clone());
        signatures.insert(path, WSignature::Struct(WStructSignature));
    }

    for item_impl in impls {
        for impl_fn in &item_impl.impl_item_fns {
            let mut path = item_impl.self_ty.clone();
            path.segments.push(WPathSegment {
                ident: impl_fn.signature.ident.clone(),
                generics: None,
            });

            let inputs = impl_fn
                .signature
                .inputs
                .iter()
                .map(|arg| arg.ty.clone())
                .collect();
            let output = impl_fn.signature.output.clone();

            signatures.insert(
                path,
                WSignature::ImplFn(WImplFnSignature { inputs, output }),
            );
        }

        for impl_type in &item_impl.impl_item_types {
            let mut path = item_impl.self_ty.clone();
            path.segments.push(WPathSegment {
                ident: impl_type.left_ident.clone(),
                generics: None,
            });

            signatures.insert(path, WSignature::ImplType(WImplTypeSignature));
        }
    }

    WSignatures::new(signatures)
}
