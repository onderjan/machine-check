use proc_macro2::TokenStream;
use syn::{
    parse2,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Expr, Item, Macro, Stmt,
};

use crate::{util::path_matches_global_names, ErrorType, MachineError};

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
            match self.process_macro(&mut stmt_macro.mac) {
                Ok(macro_result) => *stmt = Stmt::Expr(macro_result, stmt_macro.semi_token),
                Err(err) => self.push_error(err),
            }
        }
        // delegate afterwards so macros are expanded from outer to inner
        visit_mut::visit_stmt_mut(self, stmt);
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Expr::Macro(expr_macro) = expr {
            match self.process_macro(&mut expr_macro.mac) {
                Ok(macro_result) => *expr = macro_result,
                Err(err) => self.push_error(err),
            }
        }
        // delegate afterwards so macros are expanded from outer to inner
        visit_mut::visit_expr_mut(self, expr);
    }
}

impl Visitor {
    fn process_macro(&self, mac: &mut Macro) -> Result<Expr, MachineError> {
        if !path_matches_global_names(&mac.path, &["machine_check", "bitmask_switch"]) {
            return Err(MachineError::new(ErrorType::UnsupportedMacro, mac.span()));
        }

        let mut tokens = TokenStream::default();
        std::mem::swap(&mut tokens, &mut mac.tokens);

        let macro_result = match machine_check_bitmask_switch::process(tokens) {
            Ok(ok) => ok,
            Err(err) => {
                return Err(MachineError::new(
                    ErrorType::MacroError(err.msg()),
                    mac.span(),
                ));
            }
        };
        let macro_result: Expr = match parse2(macro_result) {
            Ok(ok) => ok,
            Err(err) => {
                return Err(MachineError::new(
                    ErrorType::MacroParseError(err),
                    mac.span(),
                ));
            }
        };
        Ok(macro_result)
    }
}
