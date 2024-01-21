use std::collections::HashMap;

use syn::{
    visit_mut::{self, VisitMut},
    Ident, ImplItem, ImplItemFn, Item, Pat, Stmt, Type,
};

use crate::{
    support::local::construct_prefixed_ident,
    util::{create_let_bare, extract_expr_ident_mut, extract_path_ident_mut},
    MachineError,
};

pub fn convert_mutable(items: &mut [Item]) -> Result<(), MachineError> {
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
    let mut local_ident_mut_counters = HashMap::new();

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
            // if mutable, do not retain the statement and insert to counters
            if pat_ident.mutability.is_some() {
                local_ident_mut_counters.insert(pat_ident.ident.clone(), (0u32, ty));
                retain_stmt = false;
            }
        }
        if retain_stmt {
            impl_item_fn.block.stmts.push(stmt);
        }
    }

    // visit
    let mut visitor = Visitor {
        local_ident_mut_counters,
        result: Ok(()),
    };
    visitor.visit_impl_item_fn_mut(impl_item_fn);
    visitor.result?;

    // add new local idents
    let mut stmts = Vec::new();
    for (local_ident, (mut_counter, ty)) in visitor.local_ident_mut_counters {
        for i in 1..=mut_counter {
            let temp_ident = construct_prefixed_ident(&format!("mut_{}", i), &local_ident);
            println!("Adding temp ident {}", temp_ident);
            stmts.push(create_let_bare(temp_ident, ty.clone()));
        }
    }
    stmts.append(&mut impl_item_fn.block.stmts);
    impl_item_fn.block.stmts = stmts;

    println!("Process fn: {}", quote::quote!(#impl_item_fn));

    Ok(())
}

struct Visitor {
    local_ident_mut_counters: HashMap<Ident, (u32, Option<Type>)>,
    result: Result<(), MachineError>,
}
impl VisitMut for Visitor {
    fn visit_expr_assign_mut(&mut self, expr_assign: &mut syn::ExprAssign) {
        println!("Visiting expr assign: {:?}", expr_assign);
        // visit right side first
        visit_mut::visit_expr_mut(self, &mut expr_assign.right);

        // if the left ident is mutable, change it to temporary
        let left_ident = extract_expr_ident_mut(&mut expr_assign.left)
            .expect("Left side of assignment should be expression");
        if let Some((mut_counter, _ty)) = self.local_ident_mut_counters.get_mut(left_ident) {
            *mut_counter = mut_counter
                .checked_add(1)
                .expect("Mutable counter should not overflow");
            let temp_ident = construct_prefixed_ident(&format!("mut_{}", mut_counter), left_ident);
            *left_ident = temp_ident.clone();
            println!("Expr assign to ident: {} -> {}", left_ident, temp_ident);
            /*if let Some(temp_ident_set) = result.get_mut(left_ident) {
                temp_ident_set.insert(temp_ident);
            } else {
                result.insert(left_ident.clone(), HashSet::from([temp_ident]));
            }*/
        }
    }

    fn visit_expr_if_mut(&mut self, i: &mut syn::ExprIf) {
        todo!()
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        println!("Visiting path: {:?}", path);
        // visit as ident if it is an ident, otherwise stop
        if let Some(ident) = extract_path_ident_mut(path) {
            self.visit_ident_mut(ident);
        };
    }

    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        // replace ident by temporary if necessary
        if let Some((mut_counter, _ty)) = self.local_ident_mut_counters.get(ident) {
            *ident = construct_prefixed_ident(&format!("mut_{}", mut_counter), ident);
        }
    }
}
