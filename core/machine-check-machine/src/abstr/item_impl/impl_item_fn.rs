use syn::{
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Block, ExprAssign, ExprBlock, ExprCall, Ident, ImplItemFn, Path, Stmt,
};
use syn_path::path;

use crate::util::{
    create_expr_call, create_expr_ident, create_expr_path, extract_else_token_block,
    extract_expr_ident, path_matches_global_names,
};

use super::MachineError;

use syn::{Expr, ExprIf};

pub fn process_impl_item_fn(impl_item_fn: &mut ImplItemFn) -> Result<(), MachineError> {
    // visit
    let mut visitor = Visitor {};
    visitor.visit_impl_item_fn_mut(impl_item_fn);

    Ok(())
}

struct Visitor {}

impl VisitMut for Visitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // propagate first
        visit_mut::visit_expr_mut(self, expr);
        // then convert
        self.convert_expr(expr);
    }
}

impl Visitor {
    fn convert_expr(&mut self, expr: &mut Expr) {
        let Expr::If(expr_if) = expr else {
            // only convert if expressions, the others can be used as-is
            return;
        };
        if matches!(*expr_if.cond, Expr::Lit(_)) {
            // literals are always true or false, no need to convert
            return;
        }
        // the branch should already be pre-processed by concrete conversion
        let Expr::Call(cond_expr_call) = expr_if.cond.as_mut() else {
            panic!("Non-literal branch condition should be a call");
        };
        let Expr::Path(cond_expr_path) = cond_expr_call.func.as_mut() else {
            panic!("Non-literal branch condition call function should be a path");
        };
        if cond_expr_path.path != path!(::mck::abstr::Test::into_bool) {
            panic!("Non-literal branch condition call function should be into_bool");
        }
        if cond_expr_call.args.len() != 1 {
            panic!("Expected into_bool call to have exactly one argument");
        }

        let if_token = expr_if.if_token;

        let condition = extract_expr_ident(&cond_expr_call.args[0])
            .expect("Condition should be either path or literal");

        let then_block = &expr_if.then_branch;

        // create a new condition in the else block
        let (else_token, else_block) =
            extract_else_token_block(&expr_if.else_branch).expect("Expected if with else block");

        // split into a block that contains two if statements with then branch for each branch of original:
        // 1. can be true
        // 2. can be false
        // in then branch, retain Taken within the statements, but eliminate NotTaken
        // in else branch, convert the Taken from then branch to NotTaken

        let can_be_true_if = self.create_branch_if(
            path!(::mck::abstr::Test::can_be_true),
            then_block,
            condition,
            cond_expr_call,
            if_token,
            else_token,
        );

        let can_be_false_if = self.create_branch_if(
            path!(::mck::abstr::Test::can_be_false),
            else_block,
            condition,
            cond_expr_call,
            if_token,
            else_token,
        );

        let outer_expr = Expr::Block(ExprBlock {
            attrs: vec![],
            label: None,
            block: Block {
                brace_token: Default::default(),
                stmts: vec![
                    Stmt::Expr(Expr::If(can_be_true_if), Some(Default::default())),
                    Stmt::Expr(Expr::If(can_be_false_if), Some(Default::default())),
                ],
            },
        });

        *expr = outer_expr;
    }

    fn create_branch_if(
        &mut self,
        cond_path: Path,
        taken_block: &Block,
        condition: &Ident,
        cond_expr_call: &ExprCall,
        if_token: syn::token::If,
        else_token: syn::token::Else,
    ) -> ExprIf {
        let can_be_true_cond = Expr::Call(ExprCall {
            attrs: cond_expr_call.attrs.clone(),
            func: Box::new(create_expr_path(cond_path)),
            paren_token: cond_expr_call.paren_token,
            args: cond_expr_call.args.clone(),
        });

        let mut taken_branch_block = taken_block.clone();
        let not_taken_branch_block =
            self.process_taken_branch_block(&mut taken_branch_block, condition);

        ExprIf {
            attrs: vec![],
            if_token,
            cond: Box::new(can_be_true_cond),
            then_branch: taken_branch_block,
            else_branch: Some((
                else_token,
                Box::new(Expr::Block(ExprBlock {
                    attrs: vec![],
                    label: None,
                    block: not_taken_branch_block,
                })),
            )),
        }
    }

    fn process_taken_branch_block(&mut self, taken_block: &mut Block, condition: &Ident) -> Block {
        // change Taken statements to MaybeTaken and also add them changed to NotTaken to else block
        // eliminate the NotTaken statements
        let mut taken_stmts = Vec::new();
        let mut not_taken_stmts = Vec::new();

        for mut stmt in taken_block.stmts.drain(..) {
            let Stmt::Expr(Expr::Assign(expr_assign), Some(semi)) = &mut stmt else {
                // is not an assignment
                // retain statement, but do nothing with it
                taken_stmts.push(stmt);
                continue;
            };
            let Expr::Call(expr_call) = expr_assign.right.as_mut() else {
                // does not assign a call result
                // retain statement, but do nothing with it
                taken_stmts.push(stmt);
                continue;
            };
            let Expr::Path(expr_path) = expr_call.func.as_mut() else {
                // internal error
                panic!("Call function should be a path");
            };

            if path_matches_global_names(&expr_path.path, &["mck", "forward", "PhiArg", "NotTaken"])
            {
                // eliminate NotTaken, do not retain the statement
                continue;
            }
            if !path_matches_global_names(&expr_path.path, &["mck", "forward", "PhiArg", "Taken"]) {
                // is not Taken nor NotTaken
                // retain statement, but do nothing with it
                taken_stmts.push(stmt);
                continue;
            }
            // retain as MaybeTaken
            let last_ident = &mut expr_path.path.segments[3].ident;
            *last_ident = Ident::new("MaybeTaken", last_ident.span());
            expr_call.args.push(create_expr_ident(condition.clone()));

            // also add as NotTaken to the else block
            let mut not_taken_expr_path = expr_path.clone();
            let not_taken_last_ident = &mut not_taken_expr_path.path.segments[3].ident;
            *not_taken_last_ident = Ident::new("NotTaken", not_taken_last_ident.span());
            // NotTaken has no arguments
            let not_taken_call = create_expr_call(Expr::Path(not_taken_expr_path), vec![]);
            let not_taken_expr_assign = ExprAssign {
                attrs: expr_assign.attrs.clone(),
                left: expr_assign.left.clone(),
                eq_token: expr_assign.eq_token,
                right: Box::new(not_taken_call),
            };
            let not_taken_stmt = Stmt::Expr(Expr::Assign(not_taken_expr_assign), Some(*semi));

            taken_stmts.push(stmt);
            not_taken_stmts.push(not_taken_stmt);
        }
        taken_block.stmts = taken_stmts;

        Block {
            brace_token: taken_block.brace_token,
            stmts: not_taken_stmts,
        }
    }
}
