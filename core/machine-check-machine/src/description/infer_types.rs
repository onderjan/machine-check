mod infer_call;
mod infer_fn;

use std::collections::HashMap;

use crate::wir::{
    WBasicType, WDescription, WGeneralType, WIdent, WImplItemFn, WItemImpl, WItemStruct,
    WPartialGeneralType, WPath, WSignature, WSpanned, WSsaLocal, WType, YInferred, YSsa,
};

use super::{Error, ErrorType, Errors};

pub fn infer_types(description: WDescription<YSsa>) -> Result<WDescription<YInferred>, Errors> {
    let mut structs = HashMap::new();
    // add structures first
    for item in description.structs.iter() {
        structs.insert(WPath::from_ident(item.ident.clone()), item.clone());
    }

    let mut inferred_impls = Vec::new();

    // main inference
    for item_impl in description.impls {
        let self_path = &item_impl.self_ty;

        let mut fn_items = Vec::new();

        for fn_item in item_impl.impl_item_fns {
            fn_items.push(infer_fn_types(fn_item, &structs, self_path));
        }

        let fn_items = Errors::flat_result(fn_items);

        inferred_impls.push(match fn_items {
            Ok(fn_items) => Ok(WItemImpl {
                self_ty: item_impl.self_ty,
                trait_: item_impl.trait_,
                impl_item_types: item_impl.impl_item_types,
                impl_item_fns: fn_items,
            }),
            Err(err) => Err(err),
        });
    }

    let inferred_impls = Errors::flat_result(inferred_impls)?;

    Ok(WDescription {
        structs: description.structs,
        impls: inferred_impls,
    })
}

fn infer_fn_types(
    mut impl_item_fn: WImplItemFn<YSsa>,
    structs: &HashMap<WPath, WItemStruct<WBasicType>>,
    self_path: &WPath,
) -> Result<WImplItemFn<YInferred>, Errors> {
    fn convert_self(ty: &mut WType<WBasicType>, self_path: &WPath) {
        if let WBasicType::Path(path) = &mut ty.inner {
            if path.matches_relative(&["Self"]) {
                *path = self_path.clone();
            }
        }
    }

    let mut local_ident_types = HashMap::new();

    // add param idents
    for fn_arg in &mut impl_item_fn.signature.inputs {
        let mut arg_ty = fn_arg.ty.clone();
        convert_self(&mut arg_ty, self_path);

        local_ident_types.insert(
            fn_arg.ident.clone(),
            WPartialGeneralType::Normal(arg_ty.clone()),
        );
    }

    // determine local idents and initial types
    for local in &mut impl_item_fn.locals {
        if let WPartialGeneralType::Normal(ty) = &mut local.ty {
            convert_self(ty, self_path);
        }
        local_ident_types.insert(local.ident.clone(), local.ty.clone());
    }

    // infer from statements
    let mut inferrer = FnInferrer {
        local_ident_types,
        structs,
    };

    // infer within a loop to allow for transitive inference
    inferrer.infer_fn_types_next(&impl_item_fn)?;

    // update the local types
    inferrer.update_local_types(impl_item_fn)
}

struct FnInferrer<'a> {
    local_ident_types: HashMap<WIdent, WPartialGeneralType<WBasicType>>,
    structs: &'a HashMap<WPath, WItemStruct<WBasicType>>,
}

impl FnInferrer<'_> {
    fn infer_fn_types_next(&mut self, impl_item_fn: &WImplItemFn<YSsa>) -> Result<(), Errors> {
        loop {
            // infer as much as we can
            let inferred_something = self.process_impl_item_fn(impl_item_fn)?;
            // return if we have not inferred anything
            if !inferred_something {
                return Ok(());
            }

            // we have some temporaries with the same or similar types as the originals
            // if the type of temporary is PhiArg, the original type will be in generics
            let mut local_temp_origs = HashMap::new();

            // iterate over the locals to find temporary originals
            // and determined original types
            for local in &impl_item_fn.locals {
                let mut local_type = None;
                // try to take the type from the inferrer
                let inferred_type = self.local_ident_types.get(&local.ident).unwrap();
                if inferred_type.is_fully_determined() {
                    local_type = Some(inferred_type.clone());
                }

                // remember that this temporary has an original with the same type
                local_temp_origs.insert(&local.ident, local.original.clone());
                // replace the original type with ours if ours is known, remember it
                if let Some(local_type) = local_type {
                    self.local_ident_types
                        .insert(local.original.clone(), local_type.clone());
                }
            }

            // iterate over locals once more to distribute the determined types of original
            for local in &impl_item_fn.locals {
                // look at if we have an original with some type
                if let Some(orig_ident) = local_temp_origs.get(&local.ident) {
                    if let Some(inferred_orig_type) = self.local_ident_types.get(orig_ident) {
                        if !matches!(inferred_orig_type, WPartialGeneralType::Unknown) {
                            let mut inferred_type = inferred_orig_type.clone();
                            // if temporary type is PhiArg, put the original type into generics
                            if let WPartialGeneralType::PhiArg(_) = &local.ty {
                                let WPartialGeneralType::Normal(normal_inferred_type) =
                                    inferred_type
                                else {
                                    panic!("Type in phi arg should be normal");
                                };
                                inferred_type =
                                    WPartialGeneralType::PhiArg(Some(normal_inferred_type));
                            }

                            // update the type of the temporary
                            self.local_ident_types
                                .insert(local.ident.clone(), inferred_type);
                        }
                    }
                }
            }
        }
    }

    fn update_local_types(
        &mut self,
        impl_item_fn: WImplItemFn<YSsa>,
    ) -> Result<WImplItemFn<YInferred>, Errors> {
        let mut errors = Vec::new();

        let mut locals = Vec::new();
        // add inferred types to the definitions
        for local in impl_item_fn.locals {
            let inferred_type = self.local_ident_types.remove(&local.ident).unwrap();

            let inferred_type = match inferred_type {
                WPartialGeneralType::Normal(ty) => Some(WGeneralType::Normal(ty)),
                WPartialGeneralType::PanicResult(Some(ty)) => Some(WGeneralType::PanicResult(ty)),
                WPartialGeneralType::PhiArg(Some(ty)) => Some(WGeneralType::PhiArg(ty)),
                _ => None,
            };

            match inferred_type {
                Some(inferred_type) => {
                    // add type
                    locals.push(WSsaLocal {
                        ident: local.ident,
                        original: local.original,
                        ty: inferred_type,
                    });
                }
                None => {
                    // inference failure
                    errors.push(Error::new(
                        ErrorType::InferenceFailure,
                        local.ident.wir_span(),
                    ));
                }
            }
        }

        Errors::iter_to_result(errors)?;

        let signature = WSignature {
            ident: impl_item_fn.signature.ident,
            inputs: impl_item_fn.signature.inputs,
            output: impl_item_fn.signature.output,
        };

        Ok(WImplItemFn {
            visibility: impl_item_fn.visibility,
            signature,
            locals,
            block: impl_item_fn.block,
            result: impl_item_fn.result,
        })
    }
}
