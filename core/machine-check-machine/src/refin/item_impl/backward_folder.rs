use std::collections::HashMap;

use proc_macro2::Span;
use syn::{spanned::Spanned, Expr};
use syn_path::path;

use crate::{
    abstr::ZAbstr,
    refin::{util::create_refine_join_expr, WRefinRightExpr, ZRefin},
    util::{
        create_expr_call, create_expr_field_ident, create_expr_field_unnamed, create_expr_path,
        create_expr_reference, create_expr_tuple, ArgType,
    },
    wir::{
        IntoSyn, WBlock, WCallArg, WExpr, WExprCall, WExprStruct, WIdent, WIfCondition,
        WIfConditionIdent, WStmt, WStmtAssign, WStmtIf,
    },
};

pub struct BackwardFolder {
    pub forward_ident_map: HashMap<WIdent, WIdent>,
    pub backward_ident_map: HashMap<WIdent, WIdent>,
    pub cloned_ident_map: HashMap<WIdent, (WIdent, bool)>,
    pub tmp_idents: Vec<WIdent>,
    next_tmp: u32,
}

impl BackwardFolder {
    pub fn new() -> Self {
        Self {
            forward_ident_map: HashMap::new(),
            backward_ident_map: HashMap::new(),
            cloned_ident_map: HashMap::new(),
            next_tmp: 0,
            tmp_idents: Vec::new(),
        }
    }

    pub fn fold_block(&mut self, block: WBlock<ZAbstr>) -> WBlock<ZRefin> {
        let mut stmts = Vec::new();
        for stmt in block.stmts.into_iter().rev() {
            let forward_stmt = self.fold_stmt(stmt);
            stmts.extend(forward_stmt);
        }
        WBlock { stmts }
    }

    fn fold_stmt(&mut self, stmt: WStmt<ZAbstr>) -> Vec<WStmt<ZRefin>> {
        match stmt {
            WStmt::Assign(stmt) => self.fold_assign(stmt),
            WStmt::If(stmt) => {
                let condition = match stmt.condition {
                    WIfCondition::Ident(condition_ident) => {
                        WIfCondition::Ident(WIfConditionIdent {
                            polarity: condition_ident.polarity,
                            ident: self
                                .right_cloned_ident(self.forward_ident(condition_ident.ident))
                                .0,
                        })
                    }
                    WIfCondition::Literal(lit) => WIfCondition::Literal(lit),
                };
                vec![WStmt::If(WStmtIf {
                    condition,
                    then_block: self.fold_block(stmt.then_block),
                    else_block: self.fold_block(stmt.else_block),
                })]
            }
        }
    }

    fn fold_assign(&mut self, stmt: WStmtAssign<ZAbstr>) -> Vec<WStmt<ZRefin>> {
        match stmt.right {
            WExpr::Move(right_ident) => {
                // join instead of assigning
                vec![self.backward_apply_join(
                    self.backward_ident(right_ident).into_syn(),
                    self.backward_ident(stmt.left).into_syn(),
                )]
            }
            WExpr::Call(call) => self.fold_call(stmt.left, call),
            WExpr::Field(right_field) => {
                // join the backward field
                let backward_later = self.backward_ident(stmt.left).into_syn();
                let backward_earlier = self.backward_ident(right_field.base);
                let backward_earlier = create_expr_field_ident(
                    backward_earlier.into_syn(),
                    right_field.member.to_syn_ident(),
                );
                vec![self.backward_apply_join(backward_earlier, backward_later)]
            }
            WExpr::Struct(right_struct) => self.fold_expr_struct(stmt.left, right_struct),
            WExpr::Reference(right_reference) => {
                // eliminate referencing
                match right_reference {
                    crate::wir::WExprReference::Ident(ident) => self.fold_assign(WStmtAssign {
                        left: stmt.left,
                        right: WExpr::Move(ident),
                    }),
                    crate::wir::WExprReference::Field(expr) => self.fold_assign(WStmtAssign {
                        left: stmt.left,
                        right: WExpr::Field(expr),
                    }),
                }
            }
            WExpr::Lit(_) => {
                // no backward propagation
                vec![]
            }
        }
    }

    fn fold_expr_struct(&mut self, left: WIdent, expr: WExprStruct) -> Vec<WStmt<ZRefin>> {
        // in the forward direction, we have moved data of all fields into struct
        // in the backward direction, we join the data of struct to all fields
        let backward_struct = self.backward_ident(left);
        let mut stmts = Vec::new();

        for (field_name, field_value) in expr.fields.into_iter() {
            let backward_field = self.backward_ident(field_value);
            // address the field name in the backward struct
            let tmp_field =
                create_expr_field_ident(backward_struct.clone().into_syn(), field_name.into());
            // join the temporary to the backward field
            stmts.push(self.backward_apply_join(backward_field.into_syn(), tmp_field));
        }
        stmts
    }

    fn fold_call(&mut self, left: WIdent, call: WExprCall) -> Vec<WStmt<ZRefin>> {
        enum Special {
            Phi,
            PhiTaken,
            PhiMaybeTaken,
            None,
        }

        let special = match call {
            WExprCall::StdClone(right) => {
                // convert specially
                return self.fold_clone_call(left, right);
            }
            WExprCall::Phi(_, _) => Special::Phi,
            WExprCall::PhiTaken(_) => Special::PhiTaken,
            WExprCall::PhiMaybeTaken(_) => Special::PhiMaybeTaken,
            WExprCall::PhiNotTaken => {
                // not taken branch does not have any effect
                return vec![];
            }
            WExprCall::PhiUninit => panic!("Unexpected phi uninit"),
            _ => Special::None,
        };

        // convert into syn
        let (mut call_fn, args) = call.call_fn_and_args();
        if call_fn.starts_with_absolute(&["mck", "forward"]) {
            call_fn.segments[1].ident.set_name(String::from("backward"));
        }

        let mut all_args_wild = true;
        for arg in &args {
            if !matches!(arg, WCallArg::Literal(_)) {
                all_args_wild = false;
            }
        }
        if all_args_wild {
            // cannot influence anything
            return vec![];
        }

        let span = Span::call_site();

        // the arguments should be a tuple of forward arguments
        // followed by the later backward argument

        let mut forward_args = Vec::new();
        let mut earlier_backward_args = Vec::new();

        for arg in args {
            match arg {
                crate::wir::WCallArg::Ident(ident) => {
                    let forward_ident = self.forward_ident(ident.clone());

                    let (forward_arg_ident, forward_was_reference) =
                        self.right_cloned_ident(forward_ident);
                    let mut forward_arg = forward_arg_ident.into_syn();
                    if forward_was_reference {
                        forward_arg = create_expr_reference(false, forward_arg);
                    }

                    forward_args.push(forward_arg);
                    earlier_backward_args.push(self.backward_ident(ident));
                }
                crate::wir::WCallArg::Literal(_) => {
                    // TODO: what to do here?
                    todo!("Literal arg in non-wild function")
                }
            }
        }

        let forward_args = create_expr_tuple(forward_args);

        let later_backward_arg = self.backward_ident(left);

        let mut backward_stmts = Vec::new();

        let backward_call_result = self.create_local_ident(span);

        // treat phi specially
        match special {
            Special::Phi => {
                // treat phi specially
                // we are using backward later twice, need to clone it
                let clone_tmp = self.create_local_ident(span);

                backward_stmts.push(WStmt::Assign(WStmtAssign {
                    left: clone_tmp.clone(),
                    right: WRefinRightExpr(create_expr_call(
                        create_expr_path(path!(::std::clone::Clone::clone)),
                        vec![(ArgType::Reference, later_backward_arg.clone().into_syn())],
                    )),
                }));

                backward_stmts.push(WStmt::Assign(WStmtAssign {
                    left: backward_call_result.clone(),
                    right: WRefinRightExpr(create_expr_tuple(vec![
                        later_backward_arg.into_syn(),
                        clone_tmp.into_syn(),
                    ])),
                }));
            }
            Special::PhiTaken => {
                backward_stmts.push(WStmt::Assign(WStmtAssign {
                    left: backward_call_result.clone(),
                    right: WRefinRightExpr(create_expr_tuple(vec![later_backward_arg
                        .clone()
                        .into_syn()])),
                }));
            }
            Special::PhiMaybeTaken => {
                // we are using backward later twice, need to clone it
                let clone_tmp = self.create_local_ident(span);
                backward_stmts.push(WStmt::Assign(WStmtAssign {
                    left: clone_tmp.clone(),
                    right: WRefinRightExpr(create_expr_call(
                        create_expr_path(path!(::std::clone::Clone::clone)),
                        vec![(ArgType::Reference, later_backward_arg.clone().into_syn())],
                    )),
                }));
                let to_condition = create_expr_call(
                    create_expr_path(path!(::mck::refin::Refine::to_condition)),
                    vec![(ArgType::Reference, later_backward_arg.clone().into_syn())],
                );
                backward_stmts.push(WStmt::Assign(WStmtAssign {
                    left: backward_call_result.clone(),
                    right: WRefinRightExpr(create_expr_tuple(vec![
                        clone_tmp.clone().into_syn(),
                        to_condition,
                    ])),
                }));
            }
            Special::None => {
                backward_stmts.push(WStmt::Assign(WStmtAssign {
                    left: backward_call_result.clone(),
                    right: WRefinRightExpr(create_expr_call(
                        create_expr_path(call_fn.into()),
                        vec![
                            (ArgType::Normal, forward_args),
                            (ArgType::Normal, later_backward_arg.into_syn()),
                        ],
                    )),
                }));
            }
        }

        // add statements that will join the backward call result tuple to earlier
        for (index, earlier_backward_arg) in earlier_backward_args.into_iter().enumerate() {
            let later_field =
                create_expr_field_unnamed(backward_call_result.clone().into_syn(), index);

            backward_stmts
                .push(self.backward_apply_join(earlier_backward_arg.into_syn(), later_field));
        }

        backward_stmts
    }

    fn fold_clone_call(&mut self, left: WIdent, right: WIdent) -> Vec<WStmt<ZRefin>> {
        // swap parameter and result
        // the parameter is a reference

        let backward_earlier = self.backward_ident(right);
        let backward_later = self.backward_ident(left);

        vec![self.backward_apply_join(backward_earlier.into_syn(), backward_later.into_syn())]
    }

    pub fn backward_apply_join(&self, earlier: Expr, later: Expr) -> WStmt<ZRefin> {
        let span = earlier.span();
        WStmt::Assign(WStmtAssign {
            left: WIdent::new(String::from("_"), span),
            right: WRefinRightExpr(create_refine_join_expr(earlier, later)),
        })
    }

    fn right_cloned_ident(&self, forward_ident: WIdent) -> (WIdent, bool) {
        if let Some((cloned_forward_ident, was_reference)) =
            self.cloned_ident_map.get(&forward_ident)
        {
            (cloned_forward_ident.clone(), *was_reference)
        } else {
            (forward_ident, false)
        }
    }

    fn forward_ident(&self, original_ident: WIdent) -> WIdent {
        if let Some(forward_ident) = self.forward_ident_map.get(&original_ident) {
            forward_ident.clone()
        } else {
            println!("Not found forward ident: {:?}", original_ident);
            original_ident
        }
    }

    pub fn backward_ident(&self, original_ident: WIdent) -> WIdent {
        if let Some(backward_ident) = self.backward_ident_map.get(&original_ident) {
            backward_ident.clone()
        } else {
            println!("Not found backward ident: {:?}", original_ident);
            original_ident
        }
    }

    fn create_local_ident(&mut self, span: Span) -> WIdent {
        let name = format!("__mck_backw_tmp_{}", self.next_tmp);
        self.next_tmp = self
            .next_tmp
            .checked_add(1)
            .expect("Temporary ident number should not overflow");
        let tmp_ident = WIdent::new(name, span);
        self.tmp_idents.push(tmp_ident.clone());
        tmp_ident
    }
}
