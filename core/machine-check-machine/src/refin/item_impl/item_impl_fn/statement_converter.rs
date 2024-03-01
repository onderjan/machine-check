use std::collections::HashMap;

use proc_macro2::Ident;
use syn::{spanned::Spanned, token::Semi, Expr, ExprBlock, ExprField, ExprIf, Stmt, Token, Type};

use crate::{
    refin::util::create_refine_join_stmt, support::rules::Rules, util::extract_else_block_mut,
    BackwardError, BackwardErrorType,
};

mod convert_call;

pub struct StatementConverter {
    pub clone_scheme: Rules,
    pub forward_scheme: Rules,
    pub backward_scheme: Rules,
    pub local_types: HashMap<Ident, Type>,
}

impl StatementConverter {
    pub(crate) fn convert_stmt(
        &self,
        backward_stmts: &mut Vec<Stmt>,
        stmt: &Stmt,
    ) -> Result<(), BackwardError> {
        // the statements should be in linear SSA form
        let mut stmt = stmt.clone();
        match stmt {
            Stmt::Local(ref mut local) => {
                // ensure it has no init
                assert!(local.init.is_none());
                // local without init has no side effects, do not convert
                Ok(())
            }
            Stmt::Expr(Expr::Path(_), Some(_)) | Stmt::Expr(Expr::Struct(_), Some(_)) => {
                // no side effects, do not convert
                Ok(())
            }
            Stmt::Expr(Expr::Block(expr_block), Some(semi)) => {
                self.convert_block(backward_stmts, expr_block, semi)
            }
            Stmt::Expr(Expr::If(expr_if), Some(semi)) => {
                self.convert_if(backward_stmts, expr_if, semi)
            }
            Stmt::Expr(Expr::Assign(assign), Some(_)) => {
                // assignment conversion is the most difficult, do it in own function
                self.convert_assign(backward_stmts, &assign.left, &assign.right)
            }
            Stmt::Expr(_, _) | Stmt::Item(_) | Stmt::Macro(_) => Err(BackwardError::new(
                BackwardErrorType::UnsupportedConstruct(String::from(
                    "Inversion of statement not supported",
                )),
                stmt.span(),
            )),
        }
    }

    pub(crate) fn convert_block(
        &self,
        backward_stmts: &mut Vec<Stmt>,
        mut expr_block: ExprBlock,
        semi: Semi,
    ) -> Result<(), BackwardError> {
        // convert the statements in reverse
        let mut forward_stmts = Vec::new();
        forward_stmts.append(&mut expr_block.block.stmts);
        forward_stmts.reverse();
        for forward_stmt in forward_stmts {
            self.convert_stmt(&mut expr_block.block.stmts, &forward_stmt)?;
        }

        // add the redone block to backward statements
        backward_stmts.push(Stmt::Expr(Expr::Block(expr_block), Some(semi)));
        Ok(())
    }

    pub(crate) fn convert_if(
        &self,
        backward_stmts: &mut Vec<Stmt>,
        mut expr_if: ExprIf,
        semi: Semi,
    ) -> Result<(), BackwardError> {
        // apply clone and forward scheme to condition
        // the condition refinement will be propagated through MaybeTaken
        self.clone_scheme.apply_to_expr(&mut expr_if.cond)?;
        self.forward_scheme.apply_to_expr(&mut expr_if.cond)?;
        // convert the then branch in reverse
        let then_stmts = &mut expr_if.then_branch.stmts;
        let mut forward_then_stmts = Vec::new();
        forward_then_stmts.append(then_stmts);
        forward_then_stmts.reverse();
        for stmt in forward_then_stmts {
            self.convert_stmt(then_stmts, &stmt)?;
        }
        // convert the else branch in reverse
        let else_block = extract_else_block_mut(&mut expr_if.else_branch)
            .expect("If expression should have an else block");
        let else_stmts = &mut else_block.stmts;
        let mut forward_else_stmts = Vec::new();
        forward_else_stmts.append(else_stmts);
        forward_else_stmts.reverse();
        for stmt in forward_else_stmts {
            self.convert_stmt(else_stmts, &stmt)?;
        }

        // add the redone if expression statement to backward statements
        backward_stmts.push(Stmt::Expr(Expr::If(expr_if), Some(semi)));
        Ok(())
    }

    pub(crate) fn convert_assign(
        &self,
        backward_stmts: &mut Vec<Stmt>,
        left: &Expr,
        right: &Expr,
    ) -> Result<(), BackwardError> {
        let Expr::Path(backward_later) = left else {
            panic!("Left-side expression should be path");
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
            _ => Err(BackwardError::new(
                BackwardErrorType::UnsupportedConstruct(String::from(
                    "Unable to convert expression",
                )),
                right.span(),
            )),
        }
    }
}
