use std::collections::HashMap;

use proc_macro2::Ident;
use syn::{
    spanned::Spanned, Expr, ExprCall, ExprField, ExprInfer, ExprPath, ExprReference, Pat, Stmt,
    Token, Type,
};

use crate::{
    refin::util::create_refine_join_stmt,
    support::struct_rules::StructRules,
    util::{
        create_expr_call, create_expr_field_unnamed, create_expr_ident, create_expr_path,
        create_expr_reference, create_expr_tuple, create_ident, create_let, create_pat_wild,
        extract_expr_ident, path_matches_global_names, ArgType,
    },
    ErrorType, MachineError,
};

use syn_path::path;

pub struct StatementConverter {
    pub clone_scheme: StructRules,
    pub forward_scheme: StructRules,
    pub backward_scheme: StructRules,
    pub local_types: HashMap<Ident, Type>,
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
                    Err(MachineError::new(
                        ErrorType::BackwardInternal(String::from(
                            "Inversion of let with initialization not supported",
                        )),
                        local.span(),
                    ))
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
                        return Err(MachineError::new(
                            ErrorType::BackwardInternal(String::from("Unexpected if without else")),
                            expr_if.span(),
                        ));
                    };
                    let Expr::Block(else_expr_block) = else_branch.as_mut() else {
                        return Err(MachineError::new(
                            ErrorType::BackwardInternal(String::from(
                                "Unexpected if with non-block else",
                            )),
                            else_branch.span(),
                        ));
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
            Stmt::Expr(_, _) | Stmt::Item(_) | Stmt::Macro(_) => Err(MachineError::new(
                ErrorType::BackwardInternal(String::from("Inversion of statement not supported")),
                stmt.span(),
            )),
        }
    }

    pub(crate) fn convert_assign(
        &self,
        backward_stmts: &mut Vec<Stmt>,
        left: &Expr,
        right: &Expr,
    ) -> Result<(), MachineError> {
        let Expr::Path(backward_later) = left else {
            return Err(MachineError::new(
                ErrorType::BackwardInternal(String::from(
                    "Inversion not implemented for left-side assignment expression",
                )),
                left.span(),
            ));
        };
        let mut backward_later = Expr::Path(backward_later.clone());
        self.backward_scheme.apply_to_expr(&mut backward_later)?;

        match right {
            Expr::Path(_) | Expr::Field(_) => {
                // join instead of assigning
                let mut earlier = right.clone();
                self.backward_scheme.apply_to_expr(&mut earlier)?;
                backward_stmts.push(create_refine_join_stmt(earlier, backward_later));
                Ok(())
            }
            Expr::Struct(right_struct) => {
                // join each struct field separately
                let mut earlier = right_struct.clone();
                self.backward_scheme.apply_to_expr_struct(&mut earlier)?;
                assert!(earlier.rest.is_none());
                for field_value in earlier.fields.into_iter() {
                    // value_expr = backward_later.member
                    let backward_later_field = Expr::Field(ExprField {
                        attrs: vec![],
                        base: Box::new(backward_later.clone()),
                        dot_token: Token![.](field_value.member.span()),
                        member: field_value.member,
                    });
                    let earlier = field_value.expr;
                    backward_stmts.push(create_refine_join_stmt(earlier, backward_later_field));
                }
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
            _ => Err(MachineError::new(
                ErrorType::BackwardInternal(String::from(
                    "Inversion not implemented for right-side assignment expression",
                )),
                right.span(),
            )),
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
                if let Expr::Reference(ref mut arg_ref) = call.args[0] {
                    let orig_call_param = extract_expr_ident(&arg_ref.expr)
                        .expect("Clone argument in reference should be ident")
                        .clone();
                    *arg_ref.expr = backward_later.clone();
                    stmts.push(create_let(orig_call_param.clone(), Expr::Call(call), None));
                } else {
                    let arg = &mut call.args[0];
                    let orig_call_param = extract_expr_ident(arg)
                        .expect("Clone argument should be ident")
                        .clone();
                    *arg = Expr::Reference(ExprReference {
                        attrs: vec![],
                        and_token: Token![&](arg.span()),
                        mutability: None,
                        expr: Box::new(backward_later.clone()),
                    });
                    stmts.push(create_let(orig_call_param.clone(), Expr::Call(call), None));
                }

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
                    return Err(MachineError::new(
                        ErrorType::BackwardInternal(String::from(
                            "Inversion not implemented for function argument type",
                        )),
                        arg.span(),
                    ));
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

            // make clone into a reference if it was a reference
            let arg_ident = extract_expr_ident(arg).expect("Call arg should be ident");
            let arg_ty = self
                .local_types
                .get(arg_ident)
                .expect("Call arg should be in local types");
            if matches!(arg_ty, Type::Reference(_)) {
                let mut arg_expr = Expr::Infer(ExprInfer {
                    attrs: vec![],
                    underscore_token: Default::default(),
                });
                std::mem::swap(&mut arg_expr, &mut forward_arg);
                forward_arg = create_expr_reference(false, arg_expr);
            }

            forward_args.push(forward_arg);

            let mut backward_arg = arg.clone();
            self.backward_scheme.apply_to_expr(&mut backward_arg)?;
            backward_args.push(backward_arg);
        }

        let forward_arg = create_expr_tuple(forward_args);
        backward_call.args.push(forward_arg);
        backward_call.args.push(backward_later.clone());

        // construct the backward statement, assigning to a temporary
        let tmp_ident = create_ident(&format!("__mck_backw_tmp_{}", stmts.len()));

        // treat phi specially
        let mut is_special = false;
        if let Expr::Path(ExprPath { path, .. }) = call.func.as_ref() {
            if path_matches_global_names(path, &["mck", "forward", "PhiArg", "phi"]) {
                // we are using backward later twice, need to clone it
                let backward_later_clone = create_expr_call(
                    create_expr_path(path!(::std::clone::Clone::clone)),
                    vec![(ArgType::Reference, backward_later.clone())],
                );

                assert!(call.args.len() == 2);
                stmts.push(create_let(
                    tmp_ident.clone(),
                    create_expr_tuple(vec![backward_later_clone, backward_later.clone()]),
                    None,
                ));
                is_special = true;
            }

            if path_matches_global_names(path, &["mck", "forward", "PhiArg", "Taken"]) {
                assert!(call.args.len() == 1);
                stmts.push(create_let(
                    tmp_ident.clone(),
                    create_expr_tuple(vec![backward_later.clone()]),
                    None,
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

                // we are using backward later twice, need to clone it
                let backward_later_clone = create_expr_call(
                    create_expr_path(path!(::std::clone::Clone::clone)),
                    vec![(ArgType::Reference, backward_later.clone())],
                );
                stmts.push(create_let(
                    tmp_ident.clone(),
                    create_expr_tuple(vec![backward_later_clone, to_condition]),
                    None,
                ));
                is_special = true;
            }
        }
        if !is_special {
            stmts.push(create_let(
                tmp_ident.clone(),
                Expr::Call(backward_call),
                None,
            ));
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
                    return Err(MachineError::new(
                        ErrorType::BackwardInternal(String::from(
                            "Backward join not implemented for function argument",
                        )),
                        backward_arg.span(),
                    ));
                }
            }
        }

        Ok(())
    }
}
