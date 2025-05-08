mod local_visitor;

use std::{collections::HashMap, ops::ControlFlow, vec};

use crate::{wir::*, ErrorType, MachineError};

use self::local_visitor::LocalVisitor;

pub fn infer_types(description: &mut WDescription) -> Result<(), MachineError> {
    let mut structs = HashMap::new();
    // add structures first
    for item in description.items.iter() {
        if let WItem::Struct(item_struct) = item {
            structs.insert(
                WPath::from_ident(item_struct.ident.clone()),
                item_struct.clone(),
            );
        }
    }

    // main inference
    for item in description.items.iter_mut() {
        if let WItem::Impl(item_impl) = item {
            let self_path = &item_impl.self_ty;
            for impl_item in item_impl.items.iter_mut() {
                if let WImplItem::Fn(impl_item_fn) = impl_item {
                    infer_fn_types(impl_item_fn, &structs, self_path)?;
                }
            }
        }
    }
    Ok(())
}

fn infer_fn_types(
    impl_item_fn: &mut WImplItemFn,
    structs: &HashMap<WPath, WItemStruct>,
    self_path: &WPath,
) -> Result<(), MachineError> {
    let mut local_ident_types = HashMap::new();

    // add param idents
    for fn_arg in &mut impl_item_fn.signature.inputs {
        let mut arg_ty = fn_arg.ty.clone();
        convert_self(&mut arg_ty, self_path);

        local_ident_types.insert(fn_arg.ident.clone(), Some(arg_ty.clone()));
    }

    // determine local idents and initial types
    for local in &mut impl_item_fn.block.locals {
        if let Some(ty) = &mut local.ty {
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
    while let ControlFlow::Continue(()) = infer_fn_types_next(&mut visitor, impl_item_fn)? {}

    // update the local types
    update_local_types(&mut visitor, impl_item_fn)?;

    Ok(())
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
    impl_item_fn: &mut WImplItemFn,
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
    for local in &impl_item_fn.block.locals {
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
    for local in &impl_item_fn.block.locals {
        // look at if we have an original with some type
        if let Some(orig_ident) = local_temp_origs.get(&local.ident) {
            if let Some(Some(orig_type)) = visitor.local_ident_types.get(orig_ident) {
                let mut inferred_type = orig_type.clone();
                // if temporary type is PhiArg, put the original type into generics
                if let Some(ty) = &local.ty {
                    if let WSimpleType::Path(type_path) = &ty.inner {
                        if type_path.matches_absolute(&["mck", "forward", "PhiArg"]) {
                            let mut with_generics = type_path.clone();
                            with_generics.segments[2].generics = Some(WGenerics {
                                leading_colon: false,
                                inner: vec![WGeneric::Type(inferred_type.inner)],
                            });
                            inferred_type = WType {
                                reference: ty.reference.clone(),
                                inner: WSimpleType::Path(with_generics),
                            };
                        }
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
    impl_item_fn: &mut WImplItemFn,
) -> Result<(), MachineError> {
    // add inferred types to the definitions
    for local in &mut impl_item_fn.block.locals {
        let inferred_type = visitor.local_ident_types.remove(&local.ident).unwrap();
        let Some(inferred_type) = inferred_type else {
            // inference failure
            return Err(MachineError::new(
                ErrorType::InferenceFailure,
                local.ident.span,
            ));
        };

        // add type
        local.ty = Some(inferred_type);
    }

    Ok(())
}

fn is_type_fully_specified(ty: &WType) -> bool {
    if let WSimpleType::Path(path) = &ty.inner {
        // panic result is not fully specified if it does not have generics
        if path.matches_absolute(&["machine_check", "internal", "PanicResult"]) {
            return path.segments[2].generics.is_some();
        }

        // phi arg is not fully specified if it does not have generics
        // however, since we never need to infer from phi arg,
        // we can always reject it
        !path.matches_absolute(&["mck", "forward", "PhiArg"])
    } else {
        true
    }
}
