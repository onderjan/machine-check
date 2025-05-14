use std::collections::{BTreeMap, BTreeSet};

use syn::Type;

use crate::wir::{IntoSyn, WDescription, WImplItemFn, WItemImpl, YSsa, YTotal};
use crate::MachineError;
use crate::{
    wir::{
        WBasicType, WBlock, WCallArg, WExpr, WExprCall, WIdent, WPartialGeneralType, WPath,
        WSignature, WSsaLocal, WStmt, WStmtAssign, WStmtIf, ZSsa,
    },
    ErrorType,
};

pub fn convert_to_ssa(
    description: WDescription<YTotal>,
) -> Result<WDescription<YSsa>, MachineError> {
    let mut impls = Vec::new();
    for item_impl in description.impls {
        let mut impl_item_fns = Vec::new();
        for impl_item_fn in item_impl.impl_item_fns {
            let impl_item_fn = process_fn(impl_item_fn)?;
            impl_item_fns.push(impl_item_fn);
        }
        impls.push(WItemImpl {
            self_ty: item_impl.self_ty,
            trait_: item_impl.trait_,
            impl_item_fns,
            impl_item_types: item_impl.impl_item_types,
        });
    }

    Ok(WDescription {
        structs: description.structs,
        impls,
    })
}

fn process_fn(impl_item_fn: WImplItemFn<YTotal>) -> Result<WImplItemFn<YSsa>, MachineError> {
    // TODO: process parameters

    // process mutable local idents
    let mut local_ident_counters = BTreeMap::new();

    for local in &impl_item_fn.locals {
        local_ident_counters.insert(
            local.ident.clone(),
            Counter {
                present: BTreeSet::new(),
                next: 0,
                ty: local.ty.clone(),
            },
        );
    }

    // visit
    let mut local_visitor = LocalVisitor {
        local_ident_counters,
        result: Ok(()),
        temps: BTreeMap::new(),
        branch_counter: 0,
        uninit_counter: 0,
    };
    local_visitor.process(impl_item_fn)
}

struct LocalVisitor {
    pub branch_counter: u32,
    pub local_ident_counters: BTreeMap<WIdent, Counter>,
    pub temps: BTreeMap<WIdent, (WIdent, WPartialGeneralType<WBasicType>)>,
    pub result: Result<(), MachineError>,
    pub uninit_counter: u32,
}

#[derive(Clone, Debug)]
struct Counter {
    pub present: BTreeSet<u32>,
    pub next: u32,
    pub ty: WPartialGeneralType<WBasicType>,
}

impl LocalVisitor {
    pub fn process(
        &mut self,
        mut impl_item_fn: WImplItemFn<YTotal>,
    ) -> Result<WImplItemFn<YSsa>, MachineError> {
        let signature = WSignature {
            ident: impl_item_fn.signature.ident,
            inputs: impl_item_fn.signature.inputs,
            output: impl_item_fn.signature.output,
        };

        self.process_block(&mut impl_item_fn.block);
        self.process_expr(&mut impl_item_fn.result.expr);
        self.process_ident(&mut impl_item_fn.result.panic_ident);

        self.result.clone()?;

        // replace locals with the ones in temps
        let mut locals = Vec::new();
        for (phi_temp_ident, (orig_ident, ty)) in self.temps.clone() {
            locals.push(WSsaLocal {
                ident: phi_temp_ident,
                original: orig_ident,
                ty,
            });
        }

        Ok(WImplItemFn {
            signature,
            locals,
            block: impl_item_fn.block,
            result: impl_item_fn.result,
        })
    }

    fn process_block(&mut self, block: &mut WBlock<ZSsa>) {
        let stmts: Vec<_> = block.stmts.drain(..).collect();
        for stmt in stmts {
            match stmt {
                crate::wir::WStmt::Assign(mut stmt) => {
                    self.process_assign(&mut stmt);
                    block.stmts.push(WStmt::Assign(stmt))
                }
                crate::wir::WStmt::If(stmt) => {
                    // allow adding new statements after if expression statements
                    block.stmts.extend(self.process_if(stmt));
                }
            }
        }
    }

    fn process_if(&mut self, mut stmt: WStmtIf<ZSsa>) -> impl Iterator<Item = WStmt<ZSsa>> {
        // process the condition if it is an identifier
        match &mut stmt.condition {
            WCallArg::Ident(ident) => self.process_ident(ident),
            WCallArg::Literal(_) => {
                // do nothing
            }
        }

        // process the branches

        let current_branch_counter = self.branch_counter;
        self.branch_counter = self
            .branch_counter
            .checked_add(1)
            .expect("Branch counter should not overflow");

        // detect the changed counters
        let base_counters = self.local_ident_counters.clone();

        // process then block, retain then counters, backtrack current counters, but keep next counters
        self.process_block(&mut stmt.then_block);
        let then_counters = self.local_ident_counters.clone();
        for (ident, counter) in self.local_ident_counters.iter_mut() {
            let base_counter = base_counters
                .get(ident)
                .expect("Then block ident should be in base counters");
            counter.present = base_counter.present.clone();
        }

        // visit else block
        self.process_block(&mut stmt.else_block);

        // phi changed idents
        let mut append_stmts = Vec::new();
        for (ident, else_counter) in self.local_ident_counters.iter_mut() {
            let ty = else_counter.ty.clone();
            let base_present = &base_counters
                .get(ident)
                .expect("Else block ident should be in base counters")
                .present;
            let then_present = &then_counters
                .get(ident)
                .expect("Else block ident should be in then counters")
                .present;
            let else_present = &mut else_counter.present;

            let last_base = base_present.last().cloned();
            let last_then = then_present.last().cloned();
            let last_else = else_present.last().cloned();

            if last_base == last_then && last_base == last_else {
                // this ident was not assigned to in either branch
                continue;
            }

            if last_then.is_none() || last_else.is_none() {
                // the ident was only assigned to in one branch and thus using it after the branch is an error
                continue;
            }

            // we cannot use the last_then and last_else temporaries, as they were only assigned to in one branch
            // create phi temps that will be taken in one branch and not taken in the other
            assert!(last_then != last_else);

            let last_then_ident = create_existing_temporary(
                &mut stmt.then_block,
                &mut self.temps,
                ident,
                last_then,
                ty.clone(),
                &mut self.uninit_counter,
            );
            let last_else_ident = create_existing_temporary(
                &mut stmt.else_block,
                &mut self.temps,
                ident,
                last_else,
                ty.clone(),
                &mut self.uninit_counter,
            );

            let phi_then_ident =
                ident.mck_prefixed(&format!("phi_then_{}", current_branch_counter));
            let phi_else_ident =
                ident.mck_prefixed(&format!("phi_else_{}", current_branch_counter));

            let ty = match ty {
                WPartialGeneralType::Unknown => None,
                WPartialGeneralType::Normal(ty) => Some(ty),
                _ => panic!("Phi-inner type should be unknown or normal"),
            };

            // phi then and else have phi arg type
            let phi_arg_type = WPartialGeneralType::PhiArg(ty);

            self.temps.insert(
                phi_then_ident.clone(),
                (ident.clone(), phi_arg_type.clone()),
            );
            self.temps
                .insert(phi_else_ident.clone(), (ident.clone(), phi_arg_type));

            // last then ident is taken in then block, but not in else block
            stmt.then_block.stmts.push(create_taken_assign(
                phi_then_ident.clone(),
                last_then_ident.clone(),
            ));
            stmt.else_block
                .stmts
                .push(create_not_taken_assign(phi_then_ident.clone()));

            // last else ident is not taken in then block, but is taken in else block
            stmt.then_block
                .stmts
                .push(create_not_taken_assign(phi_else_ident.clone()));
            stmt.else_block
                .stmts
                .push(create_taken_assign(phi_else_ident.clone(), last_else_ident));

            // create temporary after the if that will phi the then and else temporaries
            let append_ident = create_new_temporary(&mut self.temps, ident, else_counter);
            let append_ident_span = append_ident.span();

            append_stmts.push(WStmt::Assign(WStmtAssign {
                left: append_ident,
                right: WExpr::Call(WExprCall {
                    fn_path: WPath::new_absolute(
                        &["mck", "forward", "PhiArg", "phi"],
                        append_ident_span,
                    ),
                    args: vec![
                        WCallArg::Ident(phi_then_ident),
                        WCallArg::Ident(phi_else_ident),
                    ],
                }),
            }));
        }
        std::iter::once(WStmt::If(stmt)).chain(append_stmts)
    }

    fn process_assign(&mut self, stmt: &mut WStmtAssign<ZSsa>) {
        // process right side first
        self.process_expr(&mut stmt.right);

        // change left to temporary if needed
        if let Some(counter) = self.local_ident_counters.get_mut(&stmt.left) {
            stmt.left = create_new_temporary(&mut self.temps, &stmt.left, counter);
        }
    }

    fn process_expr<T: IntoSyn<Type>>(&mut self, expr: &mut WExpr<T>) {
        match expr {
            WExpr::Move(ident) => self.process_ident(ident),
            WExpr::Call(expr) => {
                for arg in &mut expr.args {
                    match arg {
                        crate::wir::WCallArg::Ident(ident) => self.process_ident(ident),
                        crate::wir::WCallArg::Literal(_) => {
                            // do nothing
                        }
                    }
                }
            }
            WExpr::Field(expr) => {
                // the inner is a field name, do not process it
                self.process_ident(&mut expr.base);
            }
            WExpr::Struct(expr) => {
                // do not process the struct name nor field names
                // only process assigned values
                for (_field_name, field_value) in &mut expr.fields {
                    self.process_ident(field_value);
                }
            }
            WExpr::Reference(expr) => {
                match expr {
                    crate::wir::WExprReference::Ident(ident) => self.process_ident(ident),
                    crate::wir::WExprReference::Field(field) => {
                        // the inner is a field name, do not process it
                        self.process_ident(&mut field.base);
                    }
                }
            }
            WExpr::Lit(_) => {
                // no idents, do nothing
            }
        }
    }

    fn process_ident(&mut self, ident: &mut WIdent) {
        // replace ident by temporary if necessary
        if let Some(counter) = self.local_ident_counters.get(ident) {
            // the variable must be assigned before being used
            let Some(current_counter) = counter.present.last() else {
                self.result = Err(MachineError::new(
                    ErrorType::IllegalConstruct(String::from(
                        "Variable used before being assigned",
                    )),
                    ident.span(),
                ));
                return;
            };
            *ident = construct_temp_ident(ident, *current_counter);
        }
    }
}

fn create_taken_assign(phi_arg_ident: WIdent, taken_ident: WIdent) -> WStmt<ZSsa> {
    let span = phi_arg_ident.span();
    WStmt::Assign(WStmtAssign {
        left: phi_arg_ident,
        right: WExpr::Call(WExprCall {
            fn_path: WPath::new_absolute(&["mck", "forward", "PhiArg", "Taken"], span),
            args: vec![WCallArg::Ident(taken_ident)],
        }),
    })
}

fn create_not_taken_assign(phi_arg_ident: WIdent) -> WStmt<ZSsa> {
    let span = phi_arg_ident.span();
    WStmt::Assign(WStmtAssign {
        left: phi_arg_ident,
        right: WExpr::Call(WExprCall {
            fn_path: WPath::new_absolute(&["mck", "forward", "PhiArg", "NotTaken"], span),
            args: vec![],
        }),
    })
}

fn create_new_temporary(
    temps: &mut BTreeMap<WIdent, (WIdent, WPartialGeneralType<WBasicType>)>,
    orig_ident: &WIdent,
    counter: &mut Counter,
) -> WIdent {
    let temp_ident = construct_temp_ident(orig_ident, counter.next);
    temps.insert(temp_ident.clone(), (orig_ident.clone(), counter.ty.clone()));

    counter.present.insert(counter.next);
    counter.next = counter
        .next
        .checked_add(1)
        .expect("Mutable counter should not overflow");
    temp_ident
}

fn create_existing_temporary(
    block: &mut WBlock<ZSsa>,
    temps: &mut BTreeMap<WIdent, (WIdent, WPartialGeneralType<WBasicType>)>,
    orig_ident: &WIdent,
    current: Option<u32>,
    ty: WPartialGeneralType<WBasicType>,
    uninit_counter: &mut u32,
) -> WIdent {
    if let Some(current) = current {
        construct_temp_ident(orig_ident, current)
    } else {
        let ident = orig_ident.mck_prefixed(&format!("uninit_{}", *uninit_counter));
        *uninit_counter += 1;
        let span = ident.span();
        let assign_stmt = WStmtAssign {
            left: ident.clone(),
            right: WExpr::Call(WExprCall {
                fn_path: WPath::new_absolute(&["mck", "concr", "Phi", "uninit"], span),
                args: vec![],
            }),
        };
        block.stmts.push(WStmt::Assign(assign_stmt));
        temps.insert(ident.clone(), (orig_ident.clone(), ty));
        ident
    }
}

fn construct_temp_ident(orig_ident: &WIdent, counter: u32) -> WIdent {
    orig_ident.mck_prefixed(&format!("ssa_{}", counter))
}
