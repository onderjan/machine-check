mod local_visitor;

use std::{collections::HashMap, ops::ControlFlow};

use crate::{wir::*, ErrorType, MachineError};

use self::local_visitor::LocalVisitor;

pub fn infer_types(
    description: WDescription<YSsa>,
) -> Result<WDescription<YInferred>, MachineError> {
    let mut structs = HashMap::new();
    // add structures first
    for item in description.structs.iter() {
        structs.insert(WPath::from_ident(item.ident.clone()), item.clone());
    }

    let mut inferred_impls = Vec::new();

    // main inference
    for item_impl in description.impls {
        let self_path = &item_impl.self_ty;
        let mut inferred_impl_items = Vec::new();
        for impl_item in item_impl.items.into_iter() {
            let impl_item = match impl_item {
                WImplItem::Fn(impl_item) => {
                    WImplItem::Fn(infer_fn_types(impl_item, &structs, self_path)?)
                }
                WImplItem::Type(impl_item) => WImplItem::Type(impl_item),
            };
            inferred_impl_items.push(impl_item);
        }
        inferred_impls.push(WItemImpl {
            self_ty: item_impl.self_ty,
            trait_: item_impl.trait_,
            items: inferred_impl_items,
        });
    }
    Ok(WDescription {
        structs: description.structs,
        impls: inferred_impls,
    })
}

fn infer_fn_types(
    mut impl_item_fn: WImplItemFn<YSsa>,
    structs: &HashMap<WPath, WItemStruct>,
    self_path: &WPath,
) -> Result<WImplItemFn<YInferred>, MachineError> {
    let mut local_ident_types = HashMap::new();

    // add param idents
    for fn_arg in &mut impl_item_fn.signature.inputs {
        let mut arg_ty = fn_arg.ty.clone();
        convert_self(&mut arg_ty, self_path);

        local_ident_types.insert(fn_arg.ident.clone(), Some(arg_ty.clone()));
    }

    // determine local idents and initial types
    for local in &mut impl_item_fn.locals {
        if let WPartialType(Some(ty)) = &mut local.ty {
            convert_self(ty, self_path);
        }
        local_ident_types.insert(local.ident.clone(), local.ty.0.clone());
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

fn convert_self(ty: &mut WType, self_path: &WPath) {
    if let WSimpleType::Path(path) = &mut ty.inner {
        if path.matches_relative(&["Self"]) {
            *path = self_path.clone();
        }
    }
}

fn infer_fn_types_next(
    visitor: &mut LocalVisitor<'_>,
    impl_item_fn: &WImplItemFn<YSsa>,
) -> Result<ControlFlow<(), ()>, MachineError> {
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
        if let Some(ty) = visitor.local_ident_types.get(&local.ident).unwrap() {
            if is_type_fully_specified(ty) {
                local_type = Some(ty.clone());
            }
        }

        // remember that this temporary has an original with the same type
        local_temp_origs.insert(&local.ident, local.original.clone());
        // replace the original type with ours if ours is known, remember it
        if let Some(local_type) = local_type {
            visitor
                .local_ident_types
                .insert(local.original.clone(), Some(local_type.clone()));
        }
    }

    // iterate over locals once more to distribute the determined types of original
    for local in &impl_item_fn.locals {
        // look at if we have an original with some type
        if let Some(orig_ident) = local_temp_origs.get(&local.ident) {
            if let Some(Some(orig_type)) = visitor.local_ident_types.get(orig_ident) {
                let mut inferred_type = orig_type.clone();
                // if temporary type is PhiArg, put the original type into generics
                if let WPartialType(Some(ty)) = &local.ty {
                    if let WSimpleType::PhiArg(_) = &ty.inner {
                        inferred_type.inner =
                            WSimpleType::PhiArg(Some(Box::new(inferred_type.inner)))
                    }
                }

                // update the type of the temporary
                visitor
                    .local_ident_types
                    .insert(local.ident.clone(), Some(inferred_type));
            }
        }
    }
    Ok(ControlFlow::Continue(()))
}

fn update_local_types(
    visitor: &mut LocalVisitor<'_>,
    impl_item_fn: WImplItemFn<YSsa>,
) -> Result<WImplItemFn<YInferred>, MachineError> {
    let mut locals = Vec::new();
    // add inferred types to the definitions
    for local in impl_item_fn.locals {
        let inferred_type = visitor.local_ident_types.remove(&local.ident).unwrap();
        let Some(inferred_type) = inferred_type else {
            // inference failure
            return Err(MachineError::new(
                ErrorType::InferenceFailure,
                local.ident.span,
            ));
        };

        // add type
        locals.push(WLocal::<YInferred> {
            ident: local.ident,
            original: local.original,
            ty: inferred_type,
        });
    }

    Ok(WImplItemFn {
        signature: impl_item_fn.signature,
        locals,
        block: impl_item_fn.block,
        result: impl_item_fn.result,
    })
}

fn is_type_fully_specified(ty: &WType) -> bool {
    match &ty.inner {
        WSimpleType::PanicResult(inner) => inner.is_some(),
        WSimpleType::PhiArg(inner) => inner.is_some(),
        _ => true,
    }
}
