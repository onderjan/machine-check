use syn::{
    parse2,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Attribute, Expr, ExprMacro, Ident, Item, Macro, Stmt,
};

use crate::{
    util::{create_expr_ident, path_matches_global_names},
    wir::WSpan,
};

use super::{Error, ErrorType};

pub struct MacroExpander {
    expanded_subproperties: Vec<Expr>,
}

impl MacroExpander {
    pub fn new() -> Self {
        Self {
            expanded_subproperties: Vec::new(),
        }
    }

    pub fn expand_macros(&mut self, items: &mut [Item]) -> Result<bool, Error> {
        let mut visitor = Visitor {
            expander: self,
            result: Ok(()),
            expanded_some_macro: false,
            expanding_property: false,
        };
        for item in items.iter_mut() {
            visitor.visit_item_mut(item);
        }

        visitor.result?;
        Ok(visitor.expanded_some_macro)
    }

    pub fn expand_property_macros(&mut self, expr: &mut Expr) -> Result<bool, Error> {
        let mut visitor = Visitor {
            expander: self,
            result: Ok(()),
            expanded_some_macro: false,
            expanding_property: true,
        };
        visitor.visit_expr_mut(expr);
        visitor.result?;
        Ok(visitor.expanded_some_macro)
    }

    pub fn into_expanded_subproperties(self) -> Vec<Expr> {
        self.expanded_subproperties
    }
}

struct Visitor<'a> {
    expander: &'a mut MacroExpander,
    expanding_property: bool,
    result: Result<(), Error>,
    expanded_some_macro: bool,
}

impl VisitMut for Visitor<'_> {
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

impl Visitor<'_> {
    fn process_macro(&mut self, mac: Macro, attrs: Vec<Attribute>) -> Result<Expr, Error> {
        if path_matches_global_names(&mac.path, &["machine_check", "bitmask_switch"]) {
            self.expanded_some_macro = true;
            return self.process_bitmask_switch(mac);
        }
        if self.expanding_property && path_matches_global_names(&mac.path, &["machine_check", "AG"])
        {
            let macro_inside_expr: Expr = mac.parse_body().map_err(|err| {
                let err_span = err.span();
                Error::new(ErrorType::MacroParseError(err), WSpan::from_span(err_span))
            })?;

            let ident = Ident::new(
                &format!(
                    "__mck_subproperty_{}",
                    self.expander.expanded_subproperties.len()
                ),
                mac.path.span(),
            );
            let expr = create_expr_ident(ident);
            self.expander.expanded_subproperties.push(macro_inside_expr);
            return Ok(expr);
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

    fn push_error(&mut self, err: Error) {
        if self.result.is_ok() {
            self.result = Err(err);
        }
    }
}
