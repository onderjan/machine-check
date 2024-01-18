use syn::{Expr, ExprCall, Pat, Stmt};

use crate::{
    util::{
        create_expr_field_unnamed, create_expr_ident, create_expr_tuple, create_ident, create_let,
        create_pat_wild, create_refine_join_stmt,
    },
    MachineError,
};

use super::struct_rules::StructRules;

pub struct BackwardConverter {
    pub forward_scheme: StructRules,
    pub backward_scheme: StructRules,
}

impl BackwardConverter {
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
            Stmt::Expr(Expr::Block(mut expr_block), Some(_)) => {
                // convert the statements
                let mut forward_stmts = Vec::new();
                forward_stmts.append(&mut expr_block.block.stmts);
                for stmt in forward_stmts {
                    self.convert_stmt(&mut expr_block.block.stmts, &stmt)?;
                }
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
            self.forward_scheme.apply_to_expr(&mut forward_arg)?;
            forward_args.push(forward_arg);

            let mut backward_arg = arg.clone();
            self.backward_scheme.apply_to_expr(&mut backward_arg)?;
            backward_args.push(backward_arg);
        }

        let forward_arg = create_expr_tuple(forward_args);
        backward_call.args.push(forward_arg);
        backward_call.args.push(backward_later);

        // construct the backward statement, assigning to a temporary
        let tmp_ident = create_ident(&format!("__mck_backw_tmp_{}", stmts.len()));

        stmts.push(create_let(tmp_ident.clone(), Expr::Call(backward_call)));

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
