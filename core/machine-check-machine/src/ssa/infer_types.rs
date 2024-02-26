mod fn_properties;
mod local_visitor;
mod type_properties;

use std::{collections::HashMap, ops::ControlFlow, vec};

use syn::{
    visit_mut::VisitMut, Ident, ImplItem, ImplItemFn, Item, ItemStruct, Meta, Pat, PatType, Path,
    Stmt, Type,
};
use syn_path::path;

use crate::{
    support::local::extract_local_ident_with_type,
    util::{
        create_path_from_ident, create_path_with_last_generic_type, create_type_path,
        extract_expr_ident, extract_pat_ident, extract_type_path, path_matches_global_names,
    },
    ErrorType, MachineError,
};

use self::{local_visitor::LocalVisitor, type_properties::is_type_inferrable};

pub fn infer_types(items: &mut [Item]) -> Result<(), MachineError> {
    let mut structs = HashMap::new();
    // add structures first
    for item in items.iter() {
        if let Item::Struct(item_struct) = item {
            structs.insert(
                create_path_from_ident(item_struct.ident.clone()),
                item_struct.clone(),
            );
        }
    }

    // main inference
    for item in items.iter_mut() {
        if let Item::Impl(item_impl) = item {
            for impl_item in item_impl.items.iter_mut() {
                if let ImplItem::Fn(impl_item_fn) = impl_item {
                    infer_fn_types(item_impl.self_ty.as_ref(), impl_item_fn, &structs)?;
                }
            }
        }
    }
    Ok(())
}

fn infer_fn_types(
    self_ty: &Type,
    impl_item_fn: &mut ImplItemFn,
    structs: &HashMap<Path, ItemStruct>,
) -> Result<(), MachineError> {
    let mut local_ident_types = HashMap::new();

    // add param idents
    for param in impl_item_fn.sig.inputs.iter() {
        match param {
            syn::FnArg::Receiver(receiver) => {
                let ident = Ident::new("self", receiver.self_token.span);
                local_ident_types.insert(ident, Some(self_ty.clone()));
            }
            syn::FnArg::Typed(param) => {
                // parameters are always typed
                let ident = extract_pat_ident(&param.pat);
                local_ident_types.insert(ident, Some(param.ty.as_ref().clone()));
            }
        }
    }

    // determine local idents and initial types
    for stmt in &impl_item_fn.block.stmts {
        // locals are guaranteed to be at the beginning only
        let Stmt::Local(local) = stmt else {
            break;
        };
        let (local_ident, local_type) = extract_local_ident_with_type(local);
        local_ident_types.insert(local_ident, local_type);
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

fn infer_fn_types_next(
    visitor: &mut LocalVisitor<'_>,
    impl_item_fn: &mut ImplItemFn,
) -> Result<ControlFlow<(), ()>, MachineError> {
    // visit first to infer as much as we can
    visitor.inferred_something = false;
    visitor.visit_impl_item_fn_mut(impl_item_fn);
    std::mem::replace(&mut visitor.result, Ok(()))?;
    if !visitor.inferred_something {
        return Ok(ControlFlow::Break(()));
    }

    // we have some temporaries with the same or similar types as the originals
    // if the type of temporary is PhiArg, the original type will be in generics
    let mut local_temp_origs = HashMap::new();

    // iterate over the locals to find temporary originals
    // and determined original types
    for stmt in &impl_item_fn.block.stmts {
        let Stmt::Local(local) = stmt else {
            // locals are guaranteed to be at the beginning only
            break;
        };
        let mut local_type = None;
        // extract type from local definition if possible
        let ident = match &local.pat {
            Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            Pat::Type(ty) => extract_pat_ident(&ty.pat),
            _ => panic!("Unexpected patttern type {:?}", local.pat),
        };
        // next, try to take the type from the visitor
        if let Some(ty) = visitor.local_ident_types.get(&ident).unwrap() {
            if is_type_inferrable(ty) {
                local_type = Some(ty.clone());
            }
        }

        // if we this is a temporary with an original, remember it
        // and replace the original type with ours if ours is known, remember it
        for attr in &local.attrs {
            if let Meta::NameValue(name_value) = &attr.meta {
                if name_value.path == path!(::mck::attr::tmp_original) {
                    let orig_ident = extract_expr_ident(&name_value.value).unwrap();
                    // remember that this temporary has an original with the same type
                    local_temp_origs.insert(ident, orig_ident.clone());
                    // replace the original type with ours if ours is known, remember it
                    if let Some(local_type) = local_type {
                        visitor
                            .local_ident_types
                            .insert(orig_ident.clone(), Some(local_type.clone()));
                    }
                    break;
                }
            }
        }
    }

    // iterate over locals once more to distribute the determined types of original
    for stmt in &mut impl_item_fn.block.stmts {
        let Stmt::Local(local) = stmt else {
            // locals are guaranteed to be at the beginning only
            break;
        };
        let (ident, ty) = extract_local_ident_with_type(local);
        // look at if we have an original with some type
        if let Some(orig_ident) = local_temp_origs.get(&ident) {
            if let Some(Some(orig_type)) = visitor.local_ident_types.get(orig_ident) {
                let mut inferred_type = orig_type.clone();
                // if temporary type is PhiArg, put the original type into generics
                if let Some(ty) = ty {
                    if let Some(ty_path) = extract_type_path(&ty) {
                        if path_matches_global_names(&ty_path, &["mck", "forward", "PhiArg"]) {
                            inferred_type = create_type_path(create_path_with_last_generic_type(
                                ty_path,
                                inferred_type,
                            ));
                        }
                    }
                }

                // update the type of the temporary
                visitor
                    .local_ident_types
                    .insert(ident.clone(), Some(inferred_type));
            }
        }
    }
    Ok(ControlFlow::Continue(()))
}

fn update_local_types(
    visitor: &mut LocalVisitor<'_>,
    impl_item_fn: &mut ImplItemFn,
) -> Result<(), MachineError> {
    // add inferred types to the definitions
    for stmt in &mut impl_item_fn.block.stmts {
        let Stmt::Local(local) = stmt else {
            break;
        };
        let ident = match &local.pat {
            Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            Pat::Type(ty) => extract_pat_ident(&ty.pat),
            _ => panic!("Unexpected patttern type {:?}", local.pat),
        };
        let inferred_type = visitor.local_ident_types.remove(&ident).unwrap();
        let Some(inferred_type) = inferred_type else {
            // inference failure
            return Err(MachineError::new(ErrorType::InferenceFailure, ident.span()));
        };
        // add type
        let mut pat = local.pat.clone();
        if let Pat::Type(pat_type) = pat {
            pat = pat_type.pat.as_ref().clone();
        }
        local.pat = Pat::Type(PatType {
            attrs: vec![],
            pat: Box::new(pat),
            colon_token: Default::default(),
            ty: Box::new(inferred_type),
        });
    }

    Ok(())
}
