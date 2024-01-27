mod local_visitor;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::{support::local::create_let_with_original, MachineError};
use syn::visit_mut::VisitMut;
use syn::{ImplItem, ImplItemFn, Item, Pat, Stmt};

pub fn convert_to_ssa(items: &mut [Item]) -> Result<(), MachineError> {
    for item in items.iter_mut() {
        if let Item::Impl(item_impl) = item {
            for impl_item in item_impl.items.iter_mut() {
                if let ImplItem::Fn(impl_item_fn) = impl_item {
                    process_fn(impl_item_fn)?;
                }
            }
        }
    }
    Ok(())
}

fn process_fn(impl_item_fn: &mut ImplItemFn) -> Result<(), MachineError> {
    // TODO: process parameters

    // process mutable local idents
    let mut local_ident_counters = BTreeMap::new();

    for stmt in Vec::from_iter(impl_item_fn.block.stmts.drain(..)) {
        let mut retain_stmt = true;
        if let Stmt::Local(local) = &stmt {
            let (pat, ty) = if let Pat::Type(pat_ty) = &local.pat {
                (pat_ty.pat.as_ref(), Some(pat_ty.ty.as_ref().clone()))
            } else {
                (&local.pat, None)
            };
            let Pat::Ident(pat_ident) = pat else {
                panic!("Unexpected non-ident pattern {:?}", pat);
            };
            // do not retain the statement and insert to counters
            local_ident_counters.insert(
                pat_ident.ident.clone(),
                local_visitor::Counter {
                    present: BTreeSet::new(),
                    next: 0,
                    ty,
                },
            );
            retain_stmt = false;
        }
        if retain_stmt {
            impl_item_fn.block.stmts.push(stmt);
        }
    }

    // visit
    let mut local_visitor = local_visitor::LocalVisitor {
        local_ident_counters,
        result: Ok(()),
        temps: BTreeMap::new(),
        branch_counter: 0,
    };
    local_visitor.visit_impl_item_fn_mut(impl_item_fn);
    local_visitor.result?;

    // add temporaries
    let mut stmts = Vec::new();
    for (phi_temp_ident, (orig_ident, ty)) in local_visitor.temps {
        stmts.push(create_let_with_original(
            phi_temp_ident,
            orig_ident,
            ty.clone(),
        ));
    }

    stmts.append(&mut impl_item_fn.block.stmts);
    impl_item_fn.block.stmts = stmts;

    Ok(())
}
