use proc_macro2::TokenStream;
use syn::{
    parse2,
    visit_mut::{self, VisitMut},
    Expr, Item, Macro, Stmt,
};

use crate::{util::path_matches_global_names, MachineError};

pub fn expand_macros(items: &mut [Item]) -> Result<(), MachineError> {
    let mut visitor = Visitor { result: Ok(()) };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }

    visitor.result
}

struct Visitor {
    result: Result<(), MachineError>,
}

impl Visitor {
    fn push_error(&mut self, err: MachineError) {
        if self.result.is_ok() {
            self.result = Err(err);
        }
    }
}

impl VisitMut for Visitor {
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        if let Stmt::Macro(stmt_macro) = stmt {
            if let Some(macro_result) = self.process_macro(&mut stmt_macro.mac) {
                *stmt = Stmt::Expr(macro_result, stmt_macro.semi_token);
            }
        }
        // delegate afterwards so macros are expanded from outer to inner
        visit_mut::visit_stmt_mut(self, stmt);
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Expr::Macro(expr_macro) = expr {
            if let Some(macro_result) = self.process_macro(&mut expr_macro.mac) {
                *expr = macro_result;
            }
        }
        // delegate afterwards so macros are expanded from outer to inner
        visit_mut::visit_expr_mut(self, expr);
    }
}

impl Visitor {
    fn process_macro(&self, mac: &mut Macro) -> Option<Expr> {
        if !path_matches_global_names(&mac.path, &["machine_check", "bitmask_switch"]) {
            return None;
        }

        let mut tokens = TokenStream::default();
        std::mem::swap(&mut tokens, &mut mac.tokens);

        let macro_result = match machine_check_bitmask_switch::process(tokens) {
            Ok(ok) => ok,
            Err(err) => panic!("Bitmask switch macro returned an error: {}", err.msg()),
        };
        let macro_result: Expr = match parse2(macro_result) {
            Ok(ok) => ok,
            Err(err) => panic!("Bitmask switch macro result could not be parsed: {}", err),
        };
        Some(macro_result)
    }
}
