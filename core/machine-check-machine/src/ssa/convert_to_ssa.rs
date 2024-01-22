use std::collections::{BTreeMap, HashMap};

use syn::{
    visit_mut::{self, VisitMut},
    Block, Expr, Ident, ImplItem, ImplItemFn, Item, Pat, Stmt, Type,
};
use syn_path::path;

use crate::{
    support::local::{construct_prefixed_ident, create_let_with_original},
    util::{
        create_assign, create_expr_call, create_expr_ident, create_expr_path, create_let_bare,
        create_path_with_last_generic_type, create_type_path, extract_else_block_mut,
        extract_expr_ident_mut, extract_path_ident_mut, ArgType,
    },
    MachineError,
};

pub fn convert_to_ssa(items: &mut [Item]) -> Result<(), MachineError> {
    for item in items.iter_mut() {
        if let Item::Impl(item_impl) = item {
            for impl_item in item_impl.items.iter_mut() {
                if let ImplItem::Fn(impl_item_fn) = impl_item {
                    process_fn(impl_item_fn)?;
                }
            }
        }
        //println!("After conversion: {}", quote::quote!(#item));
    }
    Ok(())
}

fn process_fn(impl_item_fn: &mut ImplItemFn) -> Result<(), MachineError> {
    // TODO: process parameters

    // process mutable local idents
    let mut local_ident_counters = HashMap::new();

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
                Counter {
                    current: None,
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
    let mut visitor = Visitor {
        local_ident_counters,
        result: Ok(()),
        temps: BTreeMap::new(),
        branch_counter: 0,
    };
    visitor.visit_impl_item_fn_mut(impl_item_fn);
    visitor.result?;

    // add temporaries
    let mut stmts = Vec::new();
    for (phi_temp_ident, (orig_ident, ty)) in visitor.temps {
        let local_stmt = if let Some(orig_ident) = orig_ident {
            create_let_with_original(phi_temp_ident, orig_ident, ty.clone())
        } else {
            create_let_bare(phi_temp_ident, ty.clone())
        };
        stmts.push(local_stmt);
    }

    stmts.append(&mut impl_item_fn.block.stmts);
    impl_item_fn.block.stmts = stmts;

    Ok(())
}

#[derive(Clone)]
struct Counter {
    current: Option<u32>,
    next: u32,
    ty: Option<Type>,
}

struct Visitor {
    branch_counter: u32,
    local_ident_counters: HashMap<Ident, Counter>,
    temps: BTreeMap<Ident, (Option<Ident>, Option<Type>)>,
    result: Result<(), MachineError>,
}
impl VisitMut for Visitor {
    fn visit_expr_assign_mut(&mut self, expr_assign: &mut syn::ExprAssign) {
        // visit right side first
        visit_mut::visit_expr_mut(self, &mut expr_assign.right);

        // if the left ident is mutable, change it to temporary
        let left_ident = extract_expr_ident_mut(&mut expr_assign.left)
            .expect("Left side of assignment should be expression");
        if let Some(counter) = self.local_ident_counters.get_mut(left_ident) {
            let temp_ident = create_new_temporary(&mut self.temps, left_ident, counter);
            *left_ident = temp_ident;
        }
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let stmts = Vec::from_iter(block.stmts.drain(..));
        // allow adding new statements after if expression statements
        for mut stmt in stmts {
            if let Stmt::Expr(Expr::If(expr_if), Some(_)) = &mut stmt {
                let extend_statements = self.process_if(expr_if);
                block.stmts.push(stmt);
                block.stmts.extend(extend_statements);
            } else {
                self.visit_stmt_mut(&mut stmt);
                block.stmts.push(stmt);
            }
        }
    }

    fn visit_expr_if_mut(&mut self, _expr_if: &mut syn::ExprIf) {
        panic!("Unexpected non-statement if expression");
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        // visit as ident if it is an ident, otherwise stop
        if let Some(ident) = extract_path_ident_mut(path) {
            self.visit_ident_mut(ident);
        };
    }

    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        // replace ident by temporary if necessary
        if let Some(counter) = self.local_ident_counters.get(ident) {
            // the variable must be used before being assigned
            let Some(current_counter) = counter.current else {
                panic!("Counter used before being assigned");
            };
            *ident = construct_temp_ident(ident, current_counter);
        }
    }
}

impl Visitor {
    fn process_if(&mut self, expr_if: &mut syn::ExprIf) -> Vec<Stmt> {
        let current_branch_counter = self.branch_counter;
        self.branch_counter = self
            .branch_counter
            .checked_add(1)
            .expect("Branch counter should not overflow");

        // visit condition
        self.visit_expr_mut(expr_if.cond.as_mut());

        let then_block = &mut expr_if.then_branch;
        let else_block =
            extract_else_block_mut(&mut expr_if.else_branch).expect("Expected if with else block");

        // detect the changed counters
        let base_counters = self.local_ident_counters.clone();

        // visit then block, retain then counters, backtrack current counters, but keep next counters
        self.visit_block_mut(then_block);
        let then_counters = self.local_ident_counters.clone();
        for (ident, counter) in self.local_ident_counters.iter_mut() {
            let base_counter = base_counters.get(ident).unwrap();
            counter.current = base_counter.current;
        }

        // visit else block
        self.visit_block_mut(else_block);

        // phi changed idents
        let mut append_stmts = Vec::new();
        for (ident, else_counter) in self.local_ident_counters.iter_mut() {
            let ty = else_counter.ty.clone();
            let last_base = base_counters.get(ident).unwrap().current;
            let last_then = then_counters.get(ident).unwrap().current;
            let last_else = else_counter.current;

            if last_base == last_then && last_base == last_else {
                // do nothing for this ident, it was not assigned to in either branch
                continue;
            }

            // we cannot use the last_then and last_else temporaries, as they were only assigned to in one branch
            // create phi temps that will be taken in one branch and not taken in the other
            assert!(last_then != last_else);

            let last_then_ident = create_existing_temporary(
                then_block,
                &mut self.temps,
                ident,
                last_then,
                ty.clone(),
            );
            let last_else_ident = create_existing_temporary(
                else_block,
                &mut self.temps,
                ident,
                last_else,
                ty.clone(),
            );

            let phi_then_ident =
                construct_prefixed_ident(&format!("phi_then_{}", current_branch_counter), ident);
            let phi_else_ident =
                construct_prefixed_ident(&format!("phi_else_{}", current_branch_counter), ident);

            // phi then and else have phi arg type
            let phi_arg_path = path!(::mck::forward::PhiArg);

            let phi_arg_type = if let Some(ty) = &ty {
                create_type_path(create_path_with_last_generic_type(phi_arg_path, ty.clone()))
            } else {
                create_type_path(phi_arg_path)
            };

            self.temps.insert(
                phi_then_ident.clone(),
                (Some(ident.clone()), Some(phi_arg_type.clone())),
            );
            self.temps.insert(
                phi_else_ident.clone(),
                (Some(ident.clone()), Some(phi_arg_type)),
            );

            // last then ident is taken in then block, but not in else block
            then_block.stmts.push(create_taken_assign(
                phi_then_ident.clone(),
                last_then_ident.clone(),
            ));
            else_block
                .stmts
                .push(create_not_taken_assign(phi_then_ident.clone()));

            // last else ident is not taken in then block, but is taken in else block
            then_block
                .stmts
                .push(create_not_taken_assign(phi_else_ident.clone()));
            else_block
                .stmts
                .push(create_taken_assign(phi_else_ident.clone(), last_else_ident));

            // create temporary after the if that will phi the then and else temporaries
            let append_ident = create_new_temporary(&mut self.temps, ident, else_counter);

            append_stmts.push(create_assign(
                append_ident,
                create_expr_call(
                    create_expr_path(path!(::mck::forward::PhiArg::phi)),
                    vec![
                        (ArgType::Normal, create_expr_ident(phi_then_ident)),
                        (ArgType::Normal, create_expr_ident(phi_else_ident)),
                    ],
                ),
                true,
            ));
        }
        append_stmts
    }
}

fn create_new_temporary(
    temps: &mut BTreeMap<Ident, (Option<Ident>, Option<Type>)>,
    orig_ident: &Ident,
    counter: &mut Counter,
) -> Ident {
    let temp_ident = construct_temp_ident(orig_ident, counter.next);
    temps.insert(
        temp_ident.clone(),
        (Some(orig_ident.clone()), counter.ty.clone()),
    );

    counter.current = Some(counter.next);
    counter.next = counter
        .next
        .checked_add(1)
        .expect("Mutable counter should not overflow");
    temp_ident
}

fn create_existing_temporary(
    block: &mut Block,
    temps: &mut BTreeMap<Ident, (Option<Ident>, Option<Type>)>,
    orig_ident: &Ident,
    current: Option<u32>,
    ty: Option<Type>,
) -> Ident {
    if let Some(current) = current {
        construct_temp_ident(orig_ident, current)
    } else {
        let ident = construct_prefixed_ident("uninit", orig_ident);
        block.stmts.push(create_assign(
            ident.clone(),
            create_expr_call(create_expr_path(path!(::mck::concr::Phi::uninit)), vec![]),
            true,
        ));
        temps.insert(ident.clone(), (Some(orig_ident.clone()), ty));
        ident
    }
}

fn construct_temp_ident(orig_ident: &Ident, counter: u32) -> Ident {
    construct_prefixed_ident(&format!("ssa_{}", counter), orig_ident)
}

fn create_taken_assign(phi_arg_ident: Ident, taken_ident: Ident) -> Stmt {
    create_assign(
        phi_arg_ident,
        create_expr_call(
            create_expr_path(path!(::mck::forward::PhiArg::Taken)),
            vec![(ArgType::Normal, create_expr_ident(taken_ident))],
        ),
        true,
    )
}

fn create_not_taken_assign(phi_arg_ident: Ident) -> Stmt {
    create_assign(
        phi_arg_ident,
        create_expr_call(
            create_expr_path(path!(::mck::forward::PhiArg::NotTaken)),
            vec![],
        ),
        true,
    )
}
