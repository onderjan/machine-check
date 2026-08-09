use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use machine_check_common::ir_common::IrReference;

use crate::into_wir::{Error, ErrorType, Errors};
use crate::wir::{
    phi_arg_item_path, WBlock, WCall, WCallArg, WExpr, WExprLowCall, WFnArg, WIdent,
    WInferredContext, WLowContext, WMckNew, WPhi, WPhiTaken, WProperty, WSignature, WSpan,
    WSpanned, WSsaLocal, WStmt, WStmtAssign, WStmtIf, WSubproperty, WSubpropertyFunc, WTypeId,
    YLowered, ZLowered, ZSsa, ZTotal,
};
use crate::wir::{WDescription, WItemFn, WItemImpl, YSsa, YTotal};

pub fn convert_description(
    ctx: &mut WLowContext,
    description: WDescription<YLowered>,
) -> Result<WDescription<YSsa>, Errors> {
    let mut impls = Vec::new();
    for item_impl in description.impls {
        let mut impl_item_fns = Vec::new();
        for impl_item_fn in item_impl.impl_item_fns {
            let (impl_item_fn, nonlocal_idents) = process_fn(ctx, impl_item_fn, &BTreeMap::new())?;
            let mut errors = Vec::new();
            for nonlocal_ident in nonlocal_idents {
                errors.push(Error::new(
                    ErrorType::UndefinedVariable(nonlocal_ident.name().to_string()),
                    WSpan::from_span(nonlocal_ident.span()),
                ));
            }
            Errors::iter_to_result(errors)?;

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

pub fn convert_property(
    ctx: &mut WLowContext,
    property: WProperty<YLowered>,
    globals: &BTreeMap<WIdent, WTypeId>,
) -> Result<WProperty<YSsa>, Errors> {
    let num_subproperties = property.subproperties.len();

    let mut converter = SubpropertyConverter {
        ctx,
        num_subproperties,
        global_ident_types: globals,
        old_subproperties: BTreeMap::from_iter(property.subproperties.into_iter().enumerate()),
        new_subproperties: BTreeMap::new(),
    };
    converter.convert_subproperty(0, &BTreeMap::new())?;

    let mut unordered_subproperties = Vec::new();

    for subproperty_index in 0..num_subproperties {
        unordered_subproperties.push(
            converter
                .new_subproperties
                .remove(&subproperty_index)
                .expect("Subproperty should be converted"),
        );
    }

    Ok(WProperty {
        subproperties: unordered_subproperties,
    })
}

struct SubpropertyConverter<'a> {
    ctx: &'a mut WLowContext,
    global_ident_types: &'a BTreeMap<WIdent, WTypeId>,
    num_subproperties: usize,
    old_subproperties: BTreeMap<usize, WSubproperty<YLowered>>,
    new_subproperties: BTreeMap<usize, WSubproperty<YSsa>>,
}
impl SubpropertyConverter<'_> {
    fn convert_subproperty(
        &mut self,
        subproperty_index: usize,
        global_rewrites: &BTreeMap<WIdent, WIdent>,
    ) -> Result<(), Errors> {
        let subproperty = self
            .old_subproperties
            .remove(&subproperty_index)
            .expect("Old subproperty should be present");

        let global_rewrites = {
            let global_rewrites = if let WSubproperty::FixedPoint(fixed_point_info) = &subproperty {
                let subproperty_ident = WIdent::new(
                    format!("__mck_subproperty_{}", subproperty_index),
                    fixed_point_info.variable.span(),
                );
                let mut global_rewrites = global_rewrites.clone();
                global_rewrites.insert(fixed_point_info.variable.clone(), subproperty_ident);
                Cow::Owned(global_rewrites)
            } else {
                Cow::Borrowed(global_rewrites)
            };

            for child_index in subproperty.children() {
                self.convert_subproperty(*child_index, &global_rewrites)?;
            }

            global_rewrites
        };

        let subproperty = match subproperty {
            WSubproperty::Func(subproperty_func) => {
                let (mut func, nonlocal_idents) =
                    process_fn(self.ctx, subproperty_func.func, &global_rewrites)?;

                // add all non-local idents to the function arguments if possible
                let mut errors = Vec::new();
                for nonlocal_ident in nonlocal_idents {
                    let ty = if let Some(ty) = self.global_ident_types.get(&nonlocal_ident) {
                        Some(ty.clone())
                    } else {
                        let mut ty = None;
                        for subproperty_index in 0..self.num_subproperties {
                            let subproperty_ident_name =
                                format!("__mck_subproperty_{}", subproperty_index);
                            if nonlocal_ident.name() == subproperty_ident_name {
                                ty = Some(self.ctx.new_bool_id());
                                break;
                            }
                        }

                        ty
                    };

                    if let Some(ty) = ty {
                        // TODO: maybe we should dereference here
                        func.signature.inputs.push(WFnArg {
                            ident: nonlocal_ident,
                            ty,
                        });
                    } else {
                        errors.push(Error::new(
                            ErrorType::UndefinedVariable(nonlocal_ident.name().to_string()),
                            WSpan::from_span(nonlocal_ident.span()),
                        ));
                    }
                }

                Errors::iter_to_result(errors)?;
                WSubproperty::Func(WSubpropertyFunc {
                    parent: subproperty_func.parent,
                    func,
                    children: subproperty_func.children,
                    display: subproperty_func.display,
                })
            }
            WSubproperty::FixedPoint(fixed_point) => WSubproperty::FixedPoint(fixed_point),
            WSubproperty::Next(next) => WSubproperty::Next(next),
        };

        self.new_subproperties
            .insert(subproperty_index, subproperty);

        Ok(())
    }
}

fn process_fn(
    ctx: &mut WLowContext,
    item_fn: WItemFn<YLowered>,
    global_rewrites: &BTreeMap<WIdent, WIdent>,
) -> Result<(WItemFn<YSsa>, BTreeSet<WIdent>), Errors> {
    // initialise local idents
    let mut local_ident_counters = BTreeMap::new();

    for local in &item_fn.locals {
        local_ident_counters.insert(
            local.ident.clone(),
            Counter {
                present: BTreeSet::new(),
                next: 0,
                ty: local.ty.clone(),
            },
        );
    }

    let arg_idents = item_fn
        .signature
        .inputs
        .iter()
        .map(|arg| arg.ident.clone())
        .collect();

    // visit
    let mut local_visitor = LocalVisitor {
        ctx,
        global_rewrites,
        arg_idents,
        local_ident_counters,
        nonlocal_idents: BTreeSet::new(),
        errors: Vec::new(),
        temps: BTreeMap::new(),
        branch_counter: 0,
    };
    let item_fn = local_visitor.process(item_fn)?;
    Ok((item_fn, local_visitor.nonlocal_idents))
}

struct LocalVisitor<'a> {
    pub ctx: &'a mut WLowContext,
    pub global_rewrites: &'a BTreeMap<WIdent, WIdent>,
    pub arg_idents: BTreeSet<WIdent>,
    pub branch_counter: u32,
    pub local_ident_counters: BTreeMap<WIdent, Counter>,
    pub nonlocal_idents: BTreeSet<WIdent>,
    pub temps: BTreeMap<WIdent, (WIdent, WTypeId)>,
    pub errors: Vec<Error>,
}

#[derive(Clone, Debug)]
struct Counter {
    pub present: BTreeSet<u32>,
    pub next: u32,
    pub ty: WTypeId,
}

impl LocalVisitor<'_> {
    pub fn process(&mut self, mut item_fn: WItemFn<YLowered>) -> Result<WItemFn<YSsa>, Errors> {
        let signature = WSignature {
            ident: item_fn.signature.ident,
            inputs: item_fn.signature.inputs,
            output: item_fn.signature.output,
        };

        let block = self.process_block(item_fn.block);
        self.process_ident(&mut item_fn.result);

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

        Ok(WItemFn {
            visibility: item_fn.visibility,
            signature,
            locals,
            block,
            result: item_fn.result,
        })
    }

    fn process_block(&mut self, block: WBlock<ZLowered>) -> WBlock<ZSsa> {
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

    fn process_if(&mut self, stmt: WStmtIf<ZLowered>) -> impl Iterator<Item = WStmt<ZSsa>> {
        // process the condition if it is an identifier
        let mut condition = stmt.condition;
        self.process_ident(&mut condition.ident);

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

            let (Some(last_then), Some(last_else)) = (last_then, last_else) else {
                // the ident was only assigned to in one branch and thus using it after the branch is an error
                continue;
            };

            // we cannot use the last_then and last_else temporaries, as they were only assigned to in one branch
            // create phi temps that will be taken in one branch and not taken in the other
            assert!(last_then != last_else);

            let last_then_ident = construct_temp_ident(ident, last_then);
            let last_else_ident = construct_temp_ident(ident, last_else);

            let phi_then_ident =
                ident.mck_prefixed(&format!("phi_then_{}", current_branch_counter));
            let phi_else_ident =
                ident.mck_prefixed(&format!("phi_else_{}", current_branch_counter));

            /*let ty = match ty {
                WPartialGeneralType::Unknown => None,
                WPartialGeneralType::Normal(ty) => Some(ty),
                _ => panic!("Phi-inner type should be unknown or normal"),
            };

            // phi then and else have phi arg type
            let phi_arg_type = WPartialGeneralType::PhiArg(ty);*/
            let phi_arg_type = self.ctx.new_phi_arg_id(ident.wir_span(), ty);

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
                condition.ident.clone(),
            ));
            else_block
                .stmts
                .push(create_not_taken_assign(phi_then_ident.clone()));

            // last else ident is not taken in then block, but is taken in else block
            then_block
                .stmts
                .push(create_not_taken_assign(phi_else_ident.clone()));
            else_block.stmts.push(create_taken_assign(
                phi_else_ident.clone(),
                last_else_ident,
                condition.ident.clone(),
            ));

            // create temporary after the if that will phi the then and else temporaries
            let append_ident = create_new_temporary(&mut self.temps, ident, else_counter);

            append_stmts.push(create_phi_call(
                append_ident,
                condition.ident.clone(),
                phi_then_ident,
                phi_else_ident,
            ));
        }
        let stmt = WStmtIf {
            condition,
            then_block,
            else_block,
        };
        std::iter::once(WStmt::If(stmt)).chain(append_stmts)
    }

    fn process_assign(&mut self, stmt: WStmtAssign<ZLowered>) -> WStmtAssign<ZSsa> {
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

    fn process_expr(&mut self, expr: &mut WExpr<WExprLowCall>) {
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
            WExpr::Lit(_, _) => {
                // no idents, do nothing
            }
        }
    }

    fn process_call(&mut self, expr: &mut WExprLowCall) {
        match expr {
            WExprLowCall::Call(call) => {
                for arg in &mut call.args {
                    match arg {
                        WCallArg::Ident(ident) => self.process_ident(ident),
                        WCallArg::Literal(_) => {
                            // do nothing
                        }
                    }
                }
            }
            WExprLowCall::MckNew(call) => match call {
                WMckNew::Bitvector(_value) => {}
                WMckNew::BitvectorArray(_ty, from) => {
                    self.process_ident(from);
                }
            },
            WExprLowCall::BooleanNew(_) => {
                // no ident, do nothing
            }
            WExprLowCall::MckUnary(call) => {
                self.process_ident(&mut call.operand);
            }
            WExprLowCall::MckBinary(call) => {
                self.process_ident(&mut call.a);
                self.process_ident(&mut call.b);
            }
            WExprLowCall::MckExt(call) => {
                self.process_ident(&mut call.from);
            }
            /*WExprLowCall::StdInto(call) => {
                self.process_ident(&mut call.from);
            }*/
            WExprLowCall::StdClone(ident) => self.process_ident(ident),
            WExprLowCall::ArrayRead(read) => {
                self.process_ident(&mut read.base);
                self.process_ident(&mut read.index);
            }
            WExprLowCall::ArrayWrite(write) => {
                self.process_ident(&mut write.base);
                self.process_ident(&mut write.index);
                self.process_ident(&mut write.element);
            }
            WExprLowCall::Phi(phi) => {
                self.process_ident(&mut phi.condition);
                self.process_ident(&mut phi.then_ident);
                self.process_ident(&mut phi.else_ident);
            }
            WExprLowCall::PhiTaken(taken) => {
                self.process_ident(&mut taken.ident);
                self.process_ident(&mut taken.condition);
            }
            WExprLowCall::PhiNotTaken => {}
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
                    ident.wir_span(),
                ));
                return;
            };
            *ident = construct_temp_ident(ident, *current_counter);
        } else {
            // rewrite first
            if let Some(rewrite_ident) = self.global_rewrites.get(ident) {
                // just replace the name and not the span
                *ident = WIdent::new(rewrite_ident.name().to_string(), ident.span());
            }

            if !self.arg_idents.contains(ident) && !self.nonlocal_idents.contains(ident) {
                self.nonlocal_idents.insert(ident.clone());
            }
        }
    }
}

fn create_phi_call(
    assigned: WIdent,
    condition: WIdent,
    then_ident: WIdent,
    else_ident: WIdent,
) -> WStmt<ZSsa> {
    let span = assigned.wir_span();

    WStmt::Assign(WStmtAssign {
        left: assigned,
        right: WExpr::Call(WExprLowCall::Phi(WPhi {
            condition,
            then_ident,
            else_ident,
        })),
    })
}

fn create_taken_assign(
    phi_arg_ident: WIdent,
    taken_ident: WIdent,
    condition_ident: WIdent,
) -> WStmt<ZSsa> {
    let span = phi_arg_ident.wir_span();

    WStmt::Assign(WStmtAssign {
        left: phi_arg_ident,
        right: WExpr::Call(WExprLowCall::PhiTaken(WPhiTaken {
            ident: taken_ident,
            condition: condition_ident,
        })),
    })
}

fn create_not_taken_assign(phi_arg_ident: WIdent) -> WStmt<ZSsa> {
    let span = phi_arg_ident.wir_span();

    WStmt::Assign(WStmtAssign {
        left: phi_arg_ident,
        right: WExpr::Call(WExprLowCall::PhiNotTaken),
    })
}

fn create_new_temporary(
    temps: &mut BTreeMap<WIdent, (WIdent, WTypeId)>,
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

fn construct_temp_ident(orig_ident: &WIdent, counter: u32) -> WIdent {
    orig_ident.mck_prefixed(&format!("ssa_{}", counter))
}
