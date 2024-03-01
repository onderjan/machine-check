use syn::{
    spanned::Spanned, Expr, ExprCall, ExprInfer, ExprPath, ExprReference, Ident, Pat, Path, Stmt,
    Token, Type,
};

use crate::{
    refin::util::create_refine_join_stmt,
    util::{
        create_expr_call, create_expr_field_unnamed, create_expr_ident, create_expr_path,
        create_expr_reference, create_expr_tuple, create_ident, create_let, create_pat_wild,
        extract_expr_ident, path_matches_global_names, ArgType,
    },
    BackwardError, BackwardErrorType,
};

use syn_path::path;

impl super::StatementConverter {
    pub(super) fn convert_call(
        &self,
        stmts: &mut Vec<Stmt>,
        backward_later: Expr,
        call: &ExprCall,
    ) -> Result<(), BackwardError> {
        let Expr::Path(ExprPath {
            path: call_func_path,
            ..
        }) = call.func.as_ref()
        else {
            panic!("Call function should be path");
        };
        if path_matches_global_names(call_func_path, &["mck", "forward", "PhiArg", "NotTaken"]) {
            if !call.args.is_empty() {
                panic!("Expected not taken args length to be empty");
            }
            // not taken branch does not have any effect, do not convert
            return Ok(());
        }

        if path_matches_global_names(call_func_path, &["std", "clone", "Clone", "clone"]) {
            return self.convert_clone_call(stmts, backward_later, call);
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
                _ => panic!("Expected path or literal call argument"),
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
        if !self.convert_phi_call(call_func_path, stmts, backward_later, call, &tmp_ident)? {
            // if the call was not phi, add a normal backward call statement
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
                    return Err(BackwardError::new(
                        BackwardErrorType::UnsupportedConstruct(String::from(
                            "Backward join not implemented for function argument",
                        )),
                        backward_arg.span(),
                    ));
                }
            }
        }

        Ok(())
    }

    pub(super) fn convert_phi_call(
        &self,
        call_func_path: &Path,
        stmts: &mut Vec<Stmt>,
        backward_later: Expr,
        call: &ExprCall,
        tmp_ident: &Ident,
    ) -> Result<bool, BackwardError> {
        let mut is_phi_call = true;
        if path_matches_global_names(call_func_path, &["mck", "forward", "PhiArg", "phi"]) {
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
        } else if path_matches_global_names(call_func_path, &["mck", "forward", "PhiArg", "Taken"])
        {
            assert!(call.args.len() == 1);
            stmts.push(create_let(
                tmp_ident.clone(),
                create_expr_tuple(vec![backward_later.clone()]),
                None,
            ));
        } else if path_matches_global_names(
            call_func_path,
            &["mck", "forward", "PhiArg", "MaybeTaken"],
        ) {
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
        } else {
            is_phi_call = false;
        }
        Ok(is_phi_call)
    }

    pub(super) fn convert_clone_call(
        &self,
        stmts: &mut Vec<Stmt>,
        backward_later: Expr,
        call: &ExprCall,
    ) -> Result<(), BackwardError> {
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

        Ok(())
    }
}
