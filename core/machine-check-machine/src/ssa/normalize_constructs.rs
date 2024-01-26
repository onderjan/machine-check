use std::vec;

use syn::{
    visit_mut::{self, VisitMut},
    Block, Expr, ExprAssign, ExprBlock, Item, Stmt,
};

use crate::{support::local::extract_local_ident_with_type, util::create_expr_ident, MachineError};

pub fn normalize_constructs(items: &mut [Item]) -> Result<(), MachineError> {
    let mut visitor = Visitor { result: Ok(()) };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }

    visitor.result
}

struct Visitor {
    result: Result<(), MachineError>,
}
impl VisitMut for Visitor {
    fn visit_block_mut(&mut self, block: &mut Block) {
        let mut processed_stmts = Vec::new();
        let num_stmts = block.stmts.len();
        for (index, stmt) in block.stmts.drain(..).enumerate() {
            match stmt {
                Stmt::Local(mut local) => {
                    let (ident, _ty) = extract_local_ident_with_type(&local);
                    // split init to assignment
                    if let Some(init) = local.init.take() {
                        if init.diverge.is_some() {
                            self.result =
                                Err(MachineError(String::from("Diverging local not supported")));
                            return;
                        }
                        let assign_stmt = Stmt::Expr(
                            Expr::Assign(ExprAssign {
                                attrs: vec![],
                                left: Box::new(create_expr_ident(ident)),
                                eq_token: init.eq_token,
                                right: init.expr,
                            }),
                            Some(local.semi_token),
                        );
                        processed_stmts.push(Stmt::Local(local));
                        processed_stmts.push(assign_stmt);
                    } else {
                        processed_stmts.push(Stmt::Local(local));
                    }
                }
                Stmt::Item(item) => {
                    // no processing here
                    processed_stmts.push(Stmt::Item(item));
                }
                Stmt::Expr(expr, mut semi) => {
                    // ensure it has semicolon if it is not the last statement
                    if semi.is_none() && index != num_stmts - 1 {
                        semi = Some(Default::default());
                    }
                    processed_stmts.push(Stmt::Expr(expr, semi));
                }
                Stmt::Macro(_stmt_macro) => {
                    self.result = Err(MachineError(String::from("Macros not supported")));
                    return;
                }
            }
        }
        block.stmts = processed_stmts;
        visit_mut::visit_block_mut(self, block);
    }

    fn visit_expr_if_mut(&mut self, expr_if: &mut syn::ExprIf) {
        // make sure it contains an else block
        if let Some((else_token, else_expr)) = expr_if.else_branch.take() {
            let else_expr = if matches!(*else_expr, Expr::Block(_)) {
                else_expr
            } else {
                // wrap the else expression inside a new block
                Box::new(Expr::Block(ExprBlock {
                    attrs: vec![],
                    label: None,
                    block: Block {
                        brace_token: Default::default(),
                        stmts: vec![Stmt::Expr(*else_expr, Some(Default::default()))],
                    },
                }))
            };
            expr_if.else_branch = Some((else_token, else_expr));
        } else {
            // create an empty else block
            expr_if.else_branch = Some((
                Default::default(),
                Box::new(Expr::Block(ExprBlock {
                    attrs: vec![],
                    label: None,
                    block: Block {
                        brace_token: Default::default(),
                        stmts: vec![],
                    },
                })),
            ));
            visit_mut::visit_expr_if_mut(self, expr_if);
        }
    }
}
