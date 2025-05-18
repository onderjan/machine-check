mod local_visitor;

use std::{collections::HashMap, ops::ControlFlow};

use crate::wir::{
    WBasicType, WDescription, WGeneralType, WImplItemFn, WItemImpl, WItemStruct,
    WPartialGeneralType, WPath, WSignature, WSsaLocal, WType, YInferred, YSsa,
};

use self::local_visitor::LocalVisitor;

use super::error::{DescriptionError, DescriptionErrorType, DescriptionErrors};

pub fn infer_types(
    description: WDescription<YSsa>,
) -> Result<WDescription<YInferred>, DescriptionErrors> {
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

        let fn_items = DescriptionErrors::flat_result(fn_items);

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

    let inferred_impls = DescriptionErrors::flat_result(inferred_impls)?;

    Ok(WDescription {
        structs: description.structs,
        impls: inferred_impls,
    })
}

fn infer_fn_types(
    mut impl_item_fn: WImplItemFn<YSsa>,
    structs: &HashMap<WPath<WBasicType>, WItemStruct<WBasicType>>,
    self_path: &WPath<WBasicType>,
) -> Result<WImplItemFn<YInferred>, DescriptionErrors> {
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
    let mut visitor = LocalVisitor {
        local_ident_types,
        structs,
        result: Ok(()),
        inferred_something: false,
    };

    // infer within a loop to allow for transitive inference
    while let ControlFlow::Continue(()) = infer_fn_types_next(&mut visitor, &impl_item_fn)? {}

    // update the local types
    update_local_types(&mut visitor, impl_item_fn)
}

fn convert_self(ty: &mut WType<WBasicType>, self_path: &WPath<WBasicType>) {
    if let WBasicType::Path(path) = &mut ty.inner {
        if path.matches_relative(&["Self"]) {
            *path = self_path.clone();
        }
    }
}

fn infer_fn_types_next(
    visitor: &mut LocalVisitor<'_>,
    impl_item_fn: &WImplItemFn<YSsa>,
) -> Result<ControlFlow<(), ()>, DescriptionError> {
    // visit first to infer as much as we can
    visitor.inferred_something = false;
    visitor.visit_impl_item_fn(impl_item_fn);
    std::mem::replace(&mut visitor.result, Ok(()))?;
    if !visitor.inferred_something {
        return Ok(ControlFlow::Break(()));
    }

    // we have some temporaries with the same or similar types as the originals
    // if the type of temporary is PhiArg, the original type will be in generics
    let mut local_temp_origs = HashMap::new();

    // iterate over the locals to find temporary originals
    // and determined original types
    for local in &impl_item_fn.locals {
        let mut local_type = None;
        // try to take the type from the visitor
        let visitor_type = visitor.local_ident_types.get(&local.ident).unwrap();
        if is_type_fully_specified(visitor_type) {
            local_type = Some(visitor_type.clone());
        }

        // remember that this temporary has an original with the same type
        local_temp_origs.insert(&local.ident, local.original.clone());
        // replace the original type with ours if ours is known, remember it
        if let Some(local_type) = local_type {
            visitor
                .local_ident_types
                .insert(local.original.clone(), local_type.clone());
        }
    }

    // iterate over locals once more to distribute the determined types of original
    for local in &impl_item_fn.locals {
        // look at if we have an original with some type
        if let Some(orig_ident) = local_temp_origs.get(&local.ident) {
            if let Some(visitor_orig_type) = visitor.local_ident_types.get(orig_ident) {
                if !matches!(visitor_orig_type, WPartialGeneralType::Unknown) {
                    let mut inferred_type = visitor_orig_type.clone();
                    // if temporary type is PhiArg, put the original type into generics
                    if let WPartialGeneralType::PhiArg(_) = &local.ty {
                        let WPartialGeneralType::Normal(normal_inferred_type) = inferred_type
                        else {
                            panic!("Type in phi arg should be normal");
                        };
                        inferred_type = WPartialGeneralType::PhiArg(Some(normal_inferred_type));
                    }

                    // update the type of the temporary
                    visitor
                        .local_ident_types
                        .insert(local.ident.clone(), inferred_type);
                }
            }
        }
    }
    Ok(ControlFlow::Continue(()))
}

fn update_local_types(
    visitor: &mut LocalVisitor<'_>,
    impl_item_fn: WImplItemFn<YSsa>,
) -> Result<WImplItemFn<YInferred>, DescriptionErrors> {
    let mut errors = Vec::new();

    let mut locals = Vec::new();
    // add inferred types to the definitions
    for local in impl_item_fn.locals {
        let inferred_type = visitor.local_ident_types.remove(&local.ident).unwrap();

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
                errors.push(DescriptionError::new(
                    DescriptionErrorType::InferenceFailure,
                    local.ident.span(),
                ));
            }
        }
    }

    DescriptionErrors::iter_to_result(errors)?;

    let signature = WSignature {
        ident: impl_item_fn.signature.ident,
        inputs: impl_item_fn.signature.inputs,
        output: impl_item_fn.signature.output,
    };

    Ok(WImplItemFn {
        signature,
        locals,
        block: impl_item_fn.block,
        result: impl_item_fn.result,
    })
}

fn is_type_fully_specified(ty: &WPartialGeneralType<WBasicType>) -> bool {
    match &ty {
        WPartialGeneralType::Unknown => false,
        WPartialGeneralType::Normal(_) => true,
        WPartialGeneralType::PanicResult(inner) => inner.is_some(),
        WPartialGeneralType::PhiArg(inner) => inner.is_some(),
    }
}
