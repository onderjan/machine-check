use anyhow::anyhow;
use syn::{Expr, ExprCall, Pat, Stmt};

use crate::machine::util::{
    create_expr_field_unnamed, create_expr_ident, create_expr_tuple, create_ident, create_let,
    create_pat_wild, create_refine_join_stmt,
};

use super::struct_rules::StructRules;

pub struct BackwardConverter {
    pub forward_scheme: StructRules,
    pub backward_scheme: StructRules,
}

impl BackwardConverter {
    pub fn convert_stmt(&self, backward_stmts: &mut Vec<Stmt>, stmt: &Stmt) -> anyhow::Result<()> {
        // the statements should be in linear SSA form
        let mut stmt = stmt.clone();
        match stmt {
            Stmt::Local(ref mut local) => {
                let Some(ref mut init) = local.init else {
                return Err(anyhow!("Inversion of non-initialized let is not supported"));
            };
                if init.diverge.is_some() {
                    return Err(anyhow!("Inversion of diverging let not supported"));
                }
                let left = &local.pat;
                let right = init.expr.as_ref();
                self.convert_let(backward_stmts, left, right)
            }
            Stmt::Expr(Expr::Path(_), Some(_)) | Stmt::Expr(Expr::Struct(_), Some(_)) => {
                // no side effects, do not convert
                Ok(())
            }
            Stmt::Expr(_, _) | Stmt::Item(_) | Stmt::Macro(_) => Err(anyhow!(
                "Inversion of statement type {:?} not supported",
                stmt
            )),
        }
    }

    pub fn convert_let(
        &self,
        inverted_stmts: &mut Vec<Stmt>,
        left: &Pat,
        right: &Expr,
    ) -> anyhow::Result<()> {
        let mut backward_later = match left {
            Pat::Ident(left_pat_ident) => create_expr_ident(left_pat_ident.ident.clone()),
            Pat::Path(left_path) => Expr::Path(left_path.clone()),
            _ => return Err(anyhow!("Inversion not implemented for pattern {:?}", left)),
        };
        self.backward_scheme.apply_to_expr(&mut backward_later)?;

        match right {
            Expr::Path(_) | Expr::Field(_) | Expr::Struct(_) => {
                // join instead of assigning
                let mut earlier = right.clone();
                self.backward_scheme.apply_to_expr(&mut earlier)?;
                inverted_stmts.push(create_refine_join_stmt(earlier, backward_later));
                Ok(())
            }
            Expr::Call(call) => self.convert_call(inverted_stmts, backward_later, call),
            _ => Err(anyhow!(
                "Inversion not implemented for expression {:?}",
                right
            )),
        }
    }

    fn convert_call(
        &self,
        stmts: &mut Vec<Stmt>,
        backward_later: Expr,
        call: &ExprCall,
    ) -> anyhow::Result<()> {
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
                    return Err(anyhow!(
                        "Inversion not implemented for function argument type {:?}",
                        arg
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
                    return Err(anyhow!(
                        "Backward join not implemented for function argument {:?}",
                        backward_arg
                    ))
                }
            }
        }

        Ok(())
    }
}
