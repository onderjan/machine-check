use syn::{Expr, ExprCall, ExprPath, Meta, Pat, Stmt};

use crate::{
    support::{local::construct_prefixed_ident, struct_rules::StructRules},
    util::{
        create_expr_call, create_expr_field_unnamed, create_expr_ident, create_expr_path,
        create_expr_tuple, create_ident, create_let, create_pat_wild, create_refine_join_stmt,
        extract_expr_ident, path_matches_global_names, ArgType,
    },
    MachineError,
};

use syn_path::path;

pub struct StatementConverter {
    pub clone_scheme: StructRules,
    pub forward_scheme: StructRules,
    pub backward_scheme: StructRules,
}

impl StatementConverter {
    pub(crate) fn convert_stmt(
        &self,
        backward_stmts: &mut Vec<Stmt>,
        stmt: &Stmt,
    ) -> Result<(), MachineError> {
        // the statements should be in linear SSA form
        let mut stmt = stmt.clone();
        match stmt {
            Stmt::Local(ref mut local) => {
                // ensure it is bare
                if local.init.is_some() {
                    Err(MachineError(String::from(
                        "Inversion of let with initialization not supported",
                    )))
                } else {
                    // no side effects, do not convert
                    Ok(())
                }
            }
            Stmt::Expr(Expr::Path(_), Some(_)) | Stmt::Expr(Expr::Struct(_), Some(_)) => {
                // no side effects, do not convert
                Ok(())
            }
            Stmt::Expr(Expr::Block(ref mut expr_block), Some(_)) => {
                // reverse and convert the statements
                let mut forward_stmts = Vec::new();
                forward_stmts.append(&mut expr_block.block.stmts);
                forward_stmts.reverse();
                for forward_stmt in forward_stmts {
                    self.convert_stmt(&mut expr_block.block.stmts, &forward_stmt)?;
                }

                // add the redone block to backward statements
                backward_stmts.push(stmt);
                Ok(())
            }
            Stmt::Expr(Expr::If(ref mut expr_if), Some(_)) => {
                // TODO: deduplicate this with block
                self.clone_scheme.apply_to_expr(&mut expr_if.cond)?;
                self.forward_scheme.apply_to_expr(&mut expr_if.cond)?;
                // reverse and convert the then branch
                {
                    let then_stmts = &mut expr_if.then_branch.stmts;
                    let mut forward_then_stmts = Vec::new();
                    forward_then_stmts.append(then_stmts);
                    forward_then_stmts.reverse();
                    for stmt in forward_then_stmts {
                        self.convert_stmt(then_stmts, &stmt)?;
                    }
                }
                // due to single-assignment requirement, there must be an else branch, convert it
                {
                    let Some((_, else_branch)) = &mut expr_if.else_branch else {
                        return Err(MachineError(format!(
                            "Unexpected if without else: {:?}",
                            expr_if
                        )));
                    };
                    let Expr::Block(else_expr_block) = else_branch.as_mut() else {
                        return Err(MachineError(format!(
                            "Unexpected if with non-block else: {:?}",
                            else_branch,
                        )));
                    };
                    let else_stmts = &mut else_expr_block.block.stmts;
                    let mut forward_else_stmts = Vec::new();
                    forward_else_stmts.append(else_stmts);
                    forward_else_stmts.reverse();
                    for stmt in forward_else_stmts {
                        self.convert_stmt(else_stmts, &stmt)?;
                    }
                }
                // TODO: propagate refinement to condition if it is abstract
                // add the redone condition to backward statements
                backward_stmts.push(stmt);

                Ok(())
            }
            Stmt::Expr(Expr::Assign(assign), Some(_)) => {
                self.convert_assign(backward_stmts, &assign.left, &assign.right)
            }
            Stmt::Expr(_, _) | Stmt::Item(_) | Stmt::Macro(_) => Err(MachineError(format!(
                "Inversion of statement type {:?} not supported",
                stmt
            ))),
        }
    }

    pub(crate) fn convert_assign(
        &self,
        backward_stmts: &mut Vec<Stmt>,
        left: &Expr,
        right: &Expr,
    ) -> Result<(), MachineError> {
        let Expr::Path(backward_later) = left else {
            return Err(MachineError(format!(
                "Inversion not implemented for left-side assignment expression: {:?}",
                left
            )));
        };
        let mut backward_later = Expr::Path(backward_later.clone());
        self.backward_scheme.apply_to_expr(&mut backward_later)?;

        match right {
            Expr::Path(_) | Expr::Field(_) | Expr::Struct(_) => {
                // join instead of assigning
                let mut earlier = right.clone();
                self.backward_scheme.apply_to_expr(&mut earlier)?;
                backward_stmts.push(create_refine_join_stmt(earlier, backward_later));
                Ok(())
            }
            Expr::Reference(expr_reference) => {
                // eliminate referencing and join instead of assigning
                let mut earlier = expr_reference.expr.as_ref().clone();
                self.backward_scheme.apply_to_expr(&mut earlier)?;
                backward_stmts.push(create_refine_join_stmt(earlier, backward_later));
                Ok(())
            }
            Expr::Call(call) => self.convert_call(backward_stmts, backward_later, call),
            _ => Err(MachineError(format!(
                "Inversion not implemented for right-side assignment expression: {:?}",
                right
            ))),
        }
    }

    fn convert_call(
        &self,
        stmts: &mut Vec<Stmt>,
        backward_later: Expr,
        call: &ExprCall,
    ) -> Result<(), MachineError> {
        if let Expr::Path(ExprPath { path, .. }) = call.func.as_ref() {
            if path_matches_global_names(path, &["mck", "forward", "PhiArg", "NotTaken"]) {
                if !call.args.is_empty() {
                    panic!("Expected not taken args length to be empty");
                }
                // do not convert
                return Ok(());
            }

            if path_matches_global_names(path, &["std", "clone", "Clone", "clone"]) {
                // swap parameter and result
                // the parameter is a reference
                let mut call = call.clone();
                let Expr::Reference(ref mut arg_ref) = call.args[0] else {
                    panic!("Clone argument should be a reference");
                };

                let orig_call_param = extract_expr_ident(&arg_ref.expr)
                    .expect("Clone argument should be ident")
                    .clone();
                *arg_ref.expr = backward_later.clone();
                stmts.push(create_let(orig_call_param.clone(), Expr::Call(call)));
                return Ok(());
            }
        }

        // early arguments are forward function arguments converted to left-side pattern
        let mut early_args = Vec::new();
        let mut all_args_wild = true;

        for arg in &call.args {
            early_args.push(match arg {
                Expr::Path(path) => {
                    all_args_wild = false;
                    Pat::Path(path.clone())
                }
                Expr::Lit(_) => create_pat_wild(),
                _ => {
                    return Err(MachineError(format!(
                        "Inversion not implemented for function argument type {:?}",
                        arg
                    )));
                }
            });
        }

        if all_args_wild {
            // do not convert
            return Ok(());
        }

        // change the function name
        let mut backward_call = call.clone();
        self.backward_scheme
            .apply_to_expr(&mut backward_call.func)?;

        // change the function parameters so that there is
        // the normal input tuple and normal output first
        // then later
        backward_call.args.clear();

        let mut forward_args = Vec::new();
        let mut backward_args = Vec::new();

        for arg in &call.args {
            let mut forward_arg = arg.clone();
            self.clone_scheme.apply_to_expr(&mut forward_arg)?;
            self.forward_scheme.apply_to_expr(&mut forward_arg)?;
            forward_args.push(forward_arg);

            let mut backward_arg = arg.clone();
            self.backward_scheme.apply_to_expr(&mut backward_arg)?;
            backward_args.push(backward_arg);
        }

        // use a clone instead of normal first argument if we have the attribute
        // TODO: remove this hack
        for attr in &call.attrs {
            if let Meta::NameValue(name_value) = &attr.meta {
                if path_matches_global_names(&name_value.path, &["mck", "attr", "refin_clone"]) {
                    // we have to convert the ident to abstract namespace
                    let ident = extract_expr_ident(&name_value.value)
                        .expect("Clone attribute should contain ident");
                    let ident = construct_prefixed_ident("abstr", ident);
                    forward_args[0] = create_expr_ident(ident);
                    break;
                }
            }
        }

        let forward_arg = create_expr_tuple(forward_args);
        backward_call.args.push(forward_arg);
        backward_call.args.push(backward_later.clone());

        // construct the backward statement, assigning to a temporary
        let tmp_ident = create_ident(&format!("__mck_backw_tmp_{}", stmts.len()));

        // treat phi specially
        let mut is_special = false;
        if let Expr::Path(ExprPath { path, .. }) = call.func.as_ref() {
            if path_matches_global_names(path, &["mck", "abstr", "Phi", "phi"]) {
                assert!(call.args.len() == 3);
                let to_condition = create_expr_call(
                    create_expr_path(path!(::mck::refin::Refine::to_condition)),
                    vec![(ArgType::Reference, backward_later.clone())],
                );
                stmts.push(create_let(
                    tmp_ident.clone(),
                    // the third argument is the condition
                    create_expr_tuple(vec![
                        backward_later.clone(),
                        backward_later.clone(),
                        to_condition,
                    ]),
                ));
                is_special = true;
            }

            if path_matches_global_names(path, &["mck", "forward", "PhiArg", "phi"]) {
                assert!(call.args.len() == 2);
                stmts.push(create_let(
                    tmp_ident.clone(),
                    create_expr_tuple(vec![backward_later.clone(), backward_later.clone()]),
                ));
                is_special = true;
            }

            if path_matches_global_names(path, &["mck", "forward", "PhiArg", "Taken"]) {
                assert!(call.args.len() == 1);
                stmts.push(create_let(
                    tmp_ident.clone(),
                    create_expr_tuple(vec![backward_later.clone()]),
                ));
                is_special = true;
            }

            if path_matches_global_names(path, &["mck", "forward", "PhiArg", "MaybeTaken"]) {
                assert!(call.args.len() == 2);
                // the second argument is the condition
                let to_condition = create_expr_call(
                    create_expr_path(path!(::mck::refin::Refine::to_condition)),
                    vec![(ArgType::Reference, backward_later.clone())],
                );
                stmts.push(create_let(
                    tmp_ident.clone(),
                    create_expr_tuple(vec![backward_later.clone(), to_condition]),
                ));
                is_special = true;
            }
        }
        if !is_special {
            stmts.push(create_let(tmp_ident.clone(), Expr::Call(backward_call)));
        }

        // we must join early instead of assigning as each early corresponds to forward argument
        // and we can use variables as forward arguments multiple times

        for (index, backward_arg) in backward_args.into_iter().enumerate() {
            match backward_arg {
                Expr::Path(_) => {
                    let right =
                        create_expr_field_unnamed(create_expr_ident(tmp_ident.clone()), index);
                    stmts.push(create_refine_join_stmt(backward_arg, right));
                }
                Expr::Lit(_) => {
                    // do nothing
                }
                _ => {
                    return Err(MachineError(format!(
                        "Backward join not implemented for function argument {:?}",
                        backward_arg
                    )))
                }
            }
        }

        Ok(())
    }
}
