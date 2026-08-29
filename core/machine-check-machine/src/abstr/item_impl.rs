use std::collections::HashMap;

use crate::{
    abstr::{WAbstrItemImplTrait, YAbstr, YAbstrIfPolarity},
    wir::{
        WBlock, WExpr, WExprLowCall, WFnSignature, WIdent, WIfCondition, WItemFn, WItemFnBody,
        WItemImpl, WItemImplTrait, WSpan, WSsaLocal, WStmt, WStmtAssign, WStmtIf, WTotalPath,
        WTotalPathSegment, YSsa,
    },
};

mod used_open;

pub fn preprocess_item_impl(item_impl: &WItemImpl<YSsa>) -> Option<WTotalPath> {
    let Some(WItemImplTrait::Machine(_)) = item_impl.trait_ else {
        return None;
    };

    let mut ty = item_impl.self_ty.clone();
    let span = WSpan::call_site();
    ty.segments.insert(
        0,
        WTotalPathSegment {
            ident: WIdent::new(String::from("super"), span),
            generics: None,
        },
    );

    Some(ty)
}

pub fn process_item_impl(
    item_impl: WItemImpl<YSsa>,
    machine_types: &[WTotalPath],
) -> Vec<WItemImpl<YAbstr>> {
    let mut impl_item_fns = Vec::new();
    for impl_item_fn in item_impl.impl_item_fns {
        impl_item_fns.push(fold_impl_item_fn(impl_item_fn));
    }

    let self_ty = item_impl.self_ty;
    let trait_ = item_impl.trait_;
    let impl_item_types = item_impl.impl_item_types;

    let mut results = Vec::new();
    for machine_type in machine_types {
        // add generics for the machine type
        let current_trait = trait_.as_ref().map(|trait_| WAbstrItemImplTrait {
            machine_type: machine_type.clone(),
            trait_: trait_.clone(),
        });

        results.push(WItemImpl {
            self_ty: self_ty.clone(),
            trait_: current_trait,
            impl_item_fns: impl_item_fns.clone(),
            impl_item_types: impl_item_types.clone(),
        });
    }

    results
}

pub fn fold_impl_item_fn(impl_item_fn: WItemFn<YSsa>) -> WItemFn<YAbstr> {
    let signature = WFnSignature {
        ident: impl_item_fn.signature.ident,
        inputs: impl_item_fn.signature.inputs,
        output: impl_item_fn.signature.output,
    };

    let mut locals_and_args_map = HashMap::new();

    for input in &signature.inputs {
        locals_and_args_map.insert(
            input.ident.clone(),
            WSsaLocal {
                ident: input.ident.clone(),
                original: input.ident.clone(),
                ty: input.ty.clone(),
            },
        );
    }

    for local in &impl_item_fn.body.locals {
        locals_and_args_map.insert(local.ident.clone(), local.clone());
    }

    let mut converter = AbstractConverter {
        /*locals: &mut impl_item_fn.locals,
        inputs: &signature.inputs,
        next_cloned_id: 0,*/
    };

    let block = converter.fold_block(impl_item_fn.body.block);

    WItemFn {
        visibility: impl_item_fn.visibility,
        signature,
        body: WItemFnBody {
            locals: impl_item_fn.body.locals,
            block,
            result: impl_item_fn.body.result,
        },
    }
}

#[derive(Debug)]
struct AbstractConverter {
    /*locals: &'a mut Vec<WSsaLocal>,
    inputs: &'a Vec<WFnArg>,
    next_cloned_id: u64,*/
}

impl AbstractConverter {
    fn fold_block(&mut self, block: WBlock<YSsa>) -> WBlock<YAbstr> {
        WBlock {
            stmts: block
                .stmts
                .into_iter()
                .flat_map(|stmt| self.fold_stmt(stmt))
                .collect(),
        }
    }

    fn fold_stmt(&mut self, stmt: WStmt<YSsa>) -> Vec<WStmt<YAbstr>> {
        match stmt {
            WStmt::Assign(stmt_assign) => {
                vec![WStmt::Assign(WStmtAssign {
                    left: stmt_assign.left,
                    right: stmt_assign.right,
                })]
            }
            WStmt::If(stmt_if) => self.fold_if(stmt_if),
        }
    }

    fn fold_if(&mut self, stmt_if: WStmtIf<YSsa>) -> Vec<WStmt<YAbstr>> {
        // split into two if statements with then branch for each branch of original:
        // 1. can be true
        // 2. can be false
        // in then branch, retain Taken within the statements, but eliminate NotTaken
        // in else branch, convert the Taken from then branch to NotTaken

        let can_be_true_stmt_if =
            self.create_branch_if(&stmt_if.condition.ident, true, stmt_if.then_block);
        let can_be_false_stmt_if =
            self.create_branch_if(&stmt_if.condition.ident, false, stmt_if.else_block);

        vec![
            WStmt::If(can_be_true_stmt_if),
            WStmt::If(can_be_false_stmt_if),
        ]
    }

    fn create_branch_if(
        &mut self,
        condition: &WIdent,
        polarity: bool,
        mut taken_block: WBlock<YSsa>,
    ) -> WStmtIf<YAbstr> {
        // first, make sure that if we use variables from above scopes,
        // they are appropriately cloned

        //let used_open_idents = used_open::used_open_idents(&taken_block);
        let mut added_start_stmts = Vec::new();

        // TODO: process used open idents
        /*for used_open_ident in used_open_idents {
            self.process_used_open_ident(&mut taken_block, &mut added_start_stmts, used_open_ident);
        }*/

        added_start_stmts.append(&mut taken_block.stmts);
        taken_block.stmts = added_start_stmts;

        // then, process the block

        let (taken_block, not_taken_block) = self.process_taken_branch_block(taken_block);

        WStmtIf {
            condition: WIfCondition {
                polarity: YAbstrIfPolarity(polarity),
                ident: condition.clone(),
            },
            then_block: taken_block,
            else_block: not_taken_block,
        }
    }

    fn process_taken_branch_block(
        &mut self,
        taken_block: WBlock<YSsa>,
    ) -> (WBlock<YAbstr>, WBlock<YAbstr>) {
        // change Taken statements to MaybeTaken and also add them changed to NotTaken to else block
        // eliminate the NotTaken statements
        let mut taken_stmts = Vec::new();
        let mut not_taken_stmts = Vec::new();

        for stmt in taken_block.stmts {
            let stmt_assign = match stmt {
                WStmt::Assign(stmt_assign) => stmt_assign,
                WStmt::If(stmt_if) => {
                    taken_stmts.extend(self.fold_if(stmt_if));
                    continue;
                }
            };

            let taken = match stmt_assign.right {
                WExpr::Call(WExprLowCall::PhiTaken(ident)) => ident,
                WExpr::Call(WExprLowCall::PhiNotTaken) => {
                    // eliminate NotTaken, do not retain the statement
                    continue;
                }
                _ => {
                    // does not concern itself with phi taken / not taken
                    // fold and retain statement in taken
                    taken_stmts.extend(self.fold_stmt(WStmt::Assign(stmt_assign)));
                    continue;
                }
            };

            // this was Taken, retain
            taken_stmts.push(WStmt::Assign(WStmtAssign {
                left: stmt_assign.left.clone(),
                right: WExpr::Call(WExprLowCall::PhiTaken(taken)),
            }));

            // also add as NotTaken to the else block
            not_taken_stmts.push(WStmt::Assign(WStmtAssign {
                left: stmt_assign.left,
                right: WExpr::Call(WExprLowCall::PhiNotTaken),
            }));
        }

        (
            WBlock { stmts: taken_stmts },
            WBlock {
                stmts: not_taken_stmts,
            },
        )
    }

    /*fn process_used_open_ident(
        &mut self,
        taken_block: &mut WBlock<YSsa>,
        added_start_stmts: &mut Vec<WStmt<YSsa>>,
        used_open_ident: WIdent,
    ) {
        /*let Some(local) = self.get_from_locals_and_idents(&used_open_ident) else {
            panic!("Not found open used {:?} in locals", used_open_ident);
        };*/

        // TODO: cloning non-copy types

        /*

        // only consider non-reference array and path-based types

        let clone_type = if let WGeneralType::Normal(WType {
            reference: IrReference::None,
            inner,
        }) = &local.ty
        {
            match inner {
                WElementaryType::Bitvector(_) | WElementaryType::Boolean => None,
                WElementaryType::Array(_) | WElementaryType::Path(_) => Some(inner.clone()),
            }
        } else {
            None
        };

        // instead of using the variable directly, clone it and use the cloned one

        let Some(clone_type) = clone_type else {
            return;
        };
        let clone_ref_ident =
            used_open_ident.mck_prefixed(&format!("clone_ref_{}", self.next_cloned_id));
        let cloned_ident = used_open_ident.mck_prefixed(&format!("cloned_{}", self.next_cloned_id));

        self.next_cloned_id += 1;

        for stmt in &mut taken_block.stmts {
            replace_stmt_ident(stmt, &used_open_ident, &cloned_ident);
        }

        let clone_ref_stmt = WStmt::Assign(WStmtAssign {
            left: clone_ref_ident.clone(),
            right: WExpr::Reference(WExprReference::Ident(used_open_ident.clone())),
        });

        let clone_stmt = WStmt::Assign(WStmtAssign {
            left: cloned_ident.clone(),
            right: WExpr::Call(WExprLowCall::StdClone(clone_ref_ident.clone())),
        });

        added_start_stmts.push(clone_ref_stmt.clone());
        added_start_stmts.push(clone_stmt.clone());

        let original_ident = local.original.clone();
        let original_ty = local.ty.clone();

        self.locals.push(WSsaLocal {
            ident: clone_ref_ident,
            original: original_ident.clone(),
            ty: WGeneralType::Normal(WType {
                reference: IrReference::Immutable,
                inner: clone_type,
            }),
        });

        self.locals.push(WSsaLocal {
            ident: cloned_ident,
            original: original_ident,
            ty: original_ty,
        });
        */
    }*/

    /*fn get_from_locals_and_idents(&self, ident: &WIdent) -> Option<WSsaLocal> {
        // TODO: make faster and nicer
        for local in self.locals.iter() {
            if &local.ident == ident {
                return Some(local.clone());
            }
        }

        for input in self.inputs {
            if &input.ident == ident {
                return Some(WSsaLocal {
                    ident: input.ident.clone(),
                    original: input.ident.clone(),
                    ty: input.ty.clone(),
                });
            }
        }

        None
    }*/
}

/*
fn replace_stmt_ident(stmt: &mut WStmt<YSsa>, original: &WIdent, replacement: &WIdent) {
    match stmt {
        WStmt::Assign(stmt_assign) => match &mut stmt_assign.right {
            WExpr::Move(ident) | WExpr::Reference(WExprReference::Ident(ident)) => {
                if ident == original {
                    *ident = replacement.clone();
                }
            }
            WExpr::Call(call) => call.replace_ident(original, replacement),
            WExpr::Field(expr_field) | WExpr::Reference(WExprReference::Field(expr_field)) => {
                if &expr_field.base == original {
                    expr_field.base = replacement.clone();
                }
            }
            WExpr::Struct(expr_struct) => {
                for (_field_name, field_value) in &mut expr_struct.fields {
                    if field_value == original {
                        *field_value = replacement.clone();
                    }
                }
            }
            WExpr::Lit(_, _) => {}
        },
        WStmt::If(stmt_if) => {
            if &stmt_if.condition.ident == original {
                stmt_if.condition.ident = replacement.clone();
            }
            for stmt in stmt_if
                .then_block
                .stmts
                .iter_mut()
                .chain(stmt_if.else_block.stmts.iter_mut())
            {
                replace_stmt_ident(stmt, original, replacement);
            }
        }
    }
}
*/
