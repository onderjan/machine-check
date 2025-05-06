use syn::{
    visit_mut::{self, VisitMut},
    Expr, ExprBlock, Item, Stmt,
};

/// Subsumes expression blocks into flat statements.
///
/// This can change scope information and so can be used only after scopes are resolved.
pub fn subsume_blocks(items: &mut [Item]) {
    let mut visitor = Visitor {};
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }
}

struct Visitor {}

impl VisitMut for Visitor {
    fn visit_impl_item_fn_mut(&mut self, impl_item: &mut syn::ImplItemFn) {
        subsume_stmt_blocks(&mut impl_item.block.stmts);
        visit_mut::visit_impl_item_fn_mut(self, impl_item);
    }

    fn visit_expr_block_mut(&mut self, expr: &mut ExprBlock) {
        visit_mut::visit_expr_block_mut(self, expr);
    }
}

fn subsume_stmt_blocks(stmts: &mut Vec<Stmt>) {
    let new_stmts: Vec<Stmt> = stmts
        .drain(..)
        .flat_map(|stmt| match stmt {
            Stmt::Expr(Expr::Block(expr_block), Some(_semi)) => expr_block.block.stmts,
            _ => vec![stmt],
        })
        .collect();
    *stmts = new_stmts;
}
