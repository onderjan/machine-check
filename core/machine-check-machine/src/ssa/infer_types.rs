mod fn_properties;
mod local_visitor;
mod type_properties;

use std::{collections::HashMap, vec};

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
    MachineError,
};

use self::{local_visitor::LocalVisitor, type_properties::is_type_standard_inferred};

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
    println!("Inferring types for function {}", impl_item_fn.sig.ident);
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

    // add local idents
    for stmt in &impl_item_fn.block.stmts {
        let Stmt::Local(local) = stmt else {
            break;
        };
        // add local ident
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
    loop {
        // TODO: remove kludge loop
        visitor.visit_impl_item_fn_mut(impl_item_fn);
        visitor.result.clone()?;
        if !visitor.inferred_something {
            break;
        }

        // infer to originals
        let mut local_temp_origs = HashMap::new();
        let mut local_orig_types = HashMap::new();

        for stmt in &impl_item_fn.block.stmts {
            let Stmt::Local(local) = stmt else {
                break;
            };
            let mut replace_type = None;
            let ident = match &local.pat {
                Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                Pat::Type(ty) => extract_pat_ident(&ty.pat),
                _ => panic!("Unexpected patttern type {:?}", local.pat),
            };
            if let Some(ty) = visitor.local_ident_types.get(&ident).unwrap() {
                if is_type_standard_inferred(ty) {
                    replace_type = Some(ty.clone());
                }
            }

            for attr in &local.attrs {
                if let Meta::NameValue(name_value) = &attr.meta {
                    if name_value.path == path!(::mck::attr::tmp_original) {
                        let orig_ident = extract_expr_ident(&name_value.value).unwrap();
                        local_temp_origs.insert(ident, orig_ident.clone());
                        // replace the original type
                        if let Some(replace_type) = replace_type {
                            local_orig_types.insert(orig_ident.clone(), replace_type);
                        }
                        break;
                    }
                }
            }
        }

        for stmt in &mut impl_item_fn.block.stmts {
            let Stmt::Local(local) = stmt else {
                break;
            };
            let (ident, ty) = match &local.pat {
                Pat::Ident(pat_ident) => (pat_ident.ident.clone(), None),
                Pat::Type(ty) => (extract_pat_ident(&ty.pat), Some(ty.ty.as_ref().clone())),
                _ => panic!("Unexpected patttern type {:?}", local.pat),
            };
            if let Some(orig_ident) = local_temp_origs.get(&ident) {
                if let Some(orig_type) = local_orig_types.get(orig_ident) {
                    let mut inferred_type = orig_type.clone();
                    if let Some(ty) = ty {
                        if let Some(ty_path) = extract_type_path(&ty) {
                            if path_matches_global_names(&ty_path, &["mck", "forward", "PhiArg"]) {
                                // put the original type into generics
                                inferred_type = create_type_path(
                                    create_path_with_last_generic_type(ty_path, inferred_type),
                                );
                            }
                        }
                    }

                    visitor
                        .local_ident_types
                        .insert(ident.clone(), Some(inferred_type));
                }
            }
        }

        visitor.inferred_something = false;
    }

    /*println!("Fn: {}", quote::quote!(#impl_item_fn));
    println!("Local ident types now:");
    for (ident, ty) in visitor.local_ident_types.iter() {
        println!("{} -> {}", ident, quote::quote!(#ty));
    }*/

    // merge local types
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
        if let Some(inferred_type) = inferred_type {
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
            })
        } else {
            // could not infer type
            return Err(MachineError(format!(
                "Could not infer type for ident {}",
                ident
            )));
        }
    }

    Ok(())
}
