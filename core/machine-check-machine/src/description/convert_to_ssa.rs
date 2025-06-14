use std::collections::{BTreeMap, BTreeSet};

use crate::wir::{
    WBasicType, WBlock, WCallArg, WExpr, WExprHighCall, WHighMckNew, WIdent, WIfCondition,
    WPartialGeneralType, WSignature, WSsaLocal, WStmt, WStmtAssign, WStmtIf, ZSsa, ZTotal,
};
use crate::wir::{WDescription, WImplItemFn, WItemImpl, YSsa, YTotal};

use super::{Error, ErrorType, Errors};

pub fn convert_to_ssa(description: WDescription<YTotal>) -> Result<WDescription<YSsa>, Errors> {
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

fn process_fn(impl_item_fn: WImplItemFn<YTotal>) -> Result<WImplItemFn<YSsa>, Errors> {
    // initialise local idents
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
        errors: Vec::new(),
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
    pub errors: Vec<Error>,
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
    ) -> Result<WImplItemFn<YSsa>, Errors> {
        let signature = WSignature {
            ident: impl_item_fn.signature.ident,
            inputs: impl_item_fn.signature.inputs,
            output: impl_item_fn.signature.output,
        };

        let block = self.process_block(impl_item_fn.block);
        self.process_ident(&mut impl_item_fn.result.result_ident);
        self.process_ident(&mut impl_item_fn.result.panic_ident);

        let mut errors = Vec::new();
        errors.append(&mut self.errors);
        Errors::iter_to_result(errors)?;

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
            visibility: impl_item_fn.visibility,
            signature,
            locals,
            block,
            result: impl_item_fn.result,
        })
    }

    fn process_block(&mut self, block: WBlock<ZTotal>) -> WBlock<ZSsa> {
        let mut stmts = Vec::new();
        for stmt in block.stmts {
            match stmt {
                WStmt::Assign(stmt) => {
                    stmts.push(WStmt::Assign(self.process_assign(stmt)));
                }
                WStmt::If(stmt) => {
                    // allow adding new statements after if expression statements
                    stmts.extend(self.process_if(stmt));
                }
            }
        }
        WBlock { stmts }
    }

    fn process_if(&mut self, stmt: WStmtIf<ZTotal>) -> impl Iterator<Item = WStmt<ZSsa>> {
        // process the condition if it is an identifier
        let mut condition = stmt.condition;
        match &mut condition {
            WIfCondition::Ident(condition_ident) => self.process_ident(&mut condition_ident.ident),
            WIfCondition::Literal(_) => {
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
        let mut then_block = self.process_block(stmt.then_block);
        let then_counters = self.local_ident_counters.clone();
        for (ident, counter) in self.local_ident_counters.iter_mut() {
            let base_counter = base_counters
                .get(ident)
                .expect("Then block ident should be in base counters");
            counter.present = base_counter.present.clone();
        }

        // visit else block
        let mut else_block = self.process_block(stmt.else_block);

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
                &mut then_block,
                &mut self.temps,
                ident,
                last_then,
                ty.clone(),
                &mut self.uninit_counter,
            );
            let last_else_ident = create_existing_temporary(
                &mut else_block,
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

            append_stmts.push(WStmt::Assign(WStmtAssign {
                left: append_ident,
                right: WExpr::Call(WExprHighCall::Phi(phi_then_ident, phi_else_ident)),
            }));
        }
        let stmt = WStmtIf {
            condition,
            then_block,
            else_block,
        };
        std::iter::once(WStmt::If(stmt)).chain(append_stmts)
    }

    fn process_assign(&mut self, stmt: WStmtAssign<ZTotal>) -> WStmtAssign<ZSsa> {
        let mut left = stmt.left;
        let mut right = stmt.right;
        // process right side first
        self.process_expr(&mut right);

        // change left to temporary if needed
        if let Some(counter) = self.local_ident_counters.get_mut(&left) {
            left = create_new_temporary(&mut self.temps, &left, counter);
        }

        WStmtAssign { left, right }
    }

    fn process_expr(&mut self, expr: &mut WExpr<WExprHighCall>) {
        match expr {
            WExpr::Move(ident) => self.process_ident(ident),
            WExpr::Call(expr) => self.process_call(expr),
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

    fn process_call(&mut self, expr: &mut WExprHighCall) {
        match expr {
            WExprHighCall::Call(call) => {
                for arg in &mut call.args {
                    match arg {
                        WCallArg::Ident(ident) => self.process_ident(ident),
                        WCallArg::Literal(_) => {
                            // do nothing
                        }
                    }
                }
            }
            WExprHighCall::MckNew(call) => {
                match call {
                    crate::wir::WHighMckNew::BitvectorArray(_type_array, ident) => {
                        self.process_ident(ident);
                    }
                    WHighMckNew::Bitvector(..)
                    | WHighMckNew::Unsigned(..)
                    | WHighMckNew::Signed(..) => {
                        // do nothing
                    }
                }
            }
            WExprHighCall::StdUnary(call) => {
                self.process_ident(&mut call.operand);
            }
            WExprHighCall::StdBinary(call) => {
                self.process_ident(&mut call.a);
                self.process_ident(&mut call.b);
            }
            WExprHighCall::MckExt(call) => {
                self.process_ident(&mut call.from);
            }
            WExprHighCall::StdInto(call) => {
                self.process_ident(&mut call.from);
            }
            WExprHighCall::StdClone(ident) => self.process_ident(ident),
            WExprHighCall::ArrayRead(read) => {
                self.process_ident(&mut read.base);
                self.process_ident(&mut read.index);
            }
            WExprHighCall::ArrayWrite(write) => {
                self.process_ident(&mut write.base);
                self.process_ident(&mut write.index);
                self.process_ident(&mut write.right);
            }
            WExprHighCall::Phi(a, b) => {
                self.process_ident(a);
                self.process_ident(b);
            }
            WExprHighCall::PhiTaken(ident) => {
                self.process_ident(ident);
            }
            WExprHighCall::PhiNotTaken => {}
            WExprHighCall::PhiUninit => {}
        }
    }

    fn process_ident(&mut self, ident: &mut WIdent) {
        // replace ident by temporary if necessary
        if let Some(counter) = self.local_ident_counters.get(ident) {
            // the variable must be assigned before being used
            let Some(current_counter) = counter.present.last() else {
                self.errors.push(Error::new(
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
    WStmt::Assign(WStmtAssign {
        left: phi_arg_ident,
        right: WExpr::Call(WExprHighCall::PhiTaken(taken_ident)),
    })
}

fn create_not_taken_assign(phi_arg_ident: WIdent) -> WStmt<ZSsa> {
    WStmt::Assign(WStmtAssign {
        left: phi_arg_ident,
        right: WExpr::Call(WExprHighCall::PhiNotTaken),
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
        let assign_stmt = WStmtAssign {
            left: ident.clone(),
            right: WExpr::Call(WExprHighCall::PhiUninit),
        };
        block.stmts.push(WStmt::Assign(assign_stmt));
        temps.insert(ident.clone(), (orig_ident.clone(), ty));
        ident
    }
}

fn construct_temp_ident(orig_ident: &WIdent, counter: u32) -> WIdent {
    orig_ident.mck_prefixed(&format!("ssa_{}", counter))
}
