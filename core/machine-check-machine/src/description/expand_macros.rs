use syn::{
    parse2,
    visit_mut::{self, VisitMut},
    Attribute, Expr, ExprMacro, Item, Macro, Stmt,
};

use crate::{util::path_matches_global_names, wir::WSpan};

use super::{Error, ErrorType};

pub struct MacroExpander {}

impl MacroExpander {
    pub fn new() -> Self {
        Self {}
    }

    pub fn expand_macros(&mut self, items: &mut [Item]) -> Result<bool, Error> {
        let mut visitor = Visitor {
            result: Ok(()),
            expanded_some_macro: false,
        };
        for item in items.iter_mut() {
            visitor.visit_item_mut(item);
        }

        visitor.result?;
        Ok(visitor.expanded_some_macro)
    }
}

struct Visitor {
    result: Result<(), Error>,
    expanded_some_macro: bool,
}

impl Visitor {
    fn push_error(&mut self, err: Error) {
        if self.result.is_ok() {
            self.result = Err(err);
        }
    }
}

impl VisitMut for Visitor {
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        // delegate first so only one macro invocation is expanded each time
        visit_mut::visit_stmt_mut(self, stmt);

        // process macros
        if let Stmt::Macro(stmt_macro) = stmt {
            match self.process_macro(stmt_macro.mac.clone(), stmt_macro.attrs.clone()) {
                Ok(macro_result) => *stmt = Stmt::Expr(macro_result, stmt_macro.semi_token),
                Err(err) => self.push_error(err),
            }
        }
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // delegate first so only one macro invocation is expanded each time
        visit_mut::visit_expr_mut(self, expr);

        // process macros
        if let Expr::Macro(expr_macro) = expr {
            match self.process_macro(expr_macro.mac.clone(), expr_macro.attrs.clone()) {
                Ok(macro_result) => *expr = macro_result,
                Err(err) => self.push_error(err),
            }
        }
    }
}

impl Visitor {
    fn process_macro(&mut self, mac: Macro, attrs: Vec<Attribute>) -> Result<Expr, Error> {
        if path_matches_global_names(&mac.path, &["machine_check", "bitmask_switch"]) {
            self.expanded_some_macro = true;
            return self.process_bitmask_switch(mac);
        }
        Ok(Expr::Macro(ExprMacro { attrs, mac }))
    }

    fn process_bitmask_switch(&self, mut mac: Macro) -> Result<Expr, Error> {
        let macro_result =
            match machine_check_bitmask_switch::process(::std::mem::take(&mut mac.tokens)) {
                Ok(ok) => ok,
                Err(err) => {
                    return Err(Error::new(
                        ErrorType::MacroError(err.msg()),
                        WSpan::from_syn(&mac),
                    ));
                }
            };
        parse2(macro_result)
            .map_err(|err| Error::new(ErrorType::MacroParseError(err), WSpan::from_syn(&mac)))
    }
}
