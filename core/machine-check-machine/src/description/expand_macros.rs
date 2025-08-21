use quote::ToTokens;
use syn::{
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Paren,
    visit_mut::{self, VisitMut},
    Attribute, BinOp, Expr, ExprBinary, ExprMacro, Ident, Item, Macro, Stmt, Token,
};
use syn_path::path;

use crate::{
    util::{create_expr_ident, path_matches_global_names},
    wir::WSpan,
};

use super::{Error, ErrorType};

pub struct ExpandedNext {
    pub universal: bool,
    pub expr: Expr,
}

pub struct ExpandedFixedPoint {
    pub universal: bool,
    pub variable: Ident,
    pub expr: Expr,
}

pub enum ExpandedSubproperty {
    Next(ExpandedNext),
    FixedPoint(ExpandedFixedPoint),
}

pub struct MacroExpander {
    expanded_subproperties: Vec<ExpandedSubproperty>,
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
        println!("Expanding property macros");
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

    pub fn into_expanded_subproperties(self) -> Vec<ExpandedSubproperty> {
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

        if !self.expanding_property {
            return Ok(Expr::Macro(ExprMacro { attrs, mac }));
        }

        let ef = path_matches_global_names(&mac.path, &["machine_check", "EF"]);
        let af = path_matches_global_names(&mac.path, &["machine_check", "AF"]);
        let eg = path_matches_global_names(&mac.path, &["machine_check", "EG"]);
        let ag = path_matches_global_names(&mac.path, &["machine_check", "AG"]);

        if ef || af || eg || ag {
            let universal = af || ag;
            let global = eg || ag;

            let inside_expr: Expr = mac.parse_body().map_err(|err| {
                let err_span = err.span();
                Error::new(ErrorType::MacroParseError(err), WSpan::from_span(err_span))
            })?;

            let mac = self.rewrite_ctl_uni(universal, global, mac, inside_expr);

            self.expanded_some_macro = true;
            return Ok(Expr::Macro(ExprMacro { attrs, mac }));
        }

        // TODO: EU/AU/ER/AR

        let ex = path_matches_global_names(&mac.path, &["machine_check", "EX"]);
        let ax = path_matches_global_names(&mac.path, &["machine_check", "AX"]);
        let lfp = path_matches_global_names(&mac.path, &["machine_check", "lfp"]);
        let gfp = path_matches_global_names(&mac.path, &["machine_check", "gfp"]);

        if ex || ax || lfp || gfp {
            let universal = ax || gfp;

            let punctuated_inside_expr: Punctuated<Expr, Token![,]> = mac
                .parse_body_with(Punctuated::parse_terminated)
                .map_err(|err| {
                    let err_span = err.span();
                    Error::new(ErrorType::MacroParseError(err), WSpan::from_span(err_span))
                })?;

            let subproperty = if ex || ax {
                if punctuated_inside_expr.len() != 1 {
                    return Err(Error::new(
                        ErrorType::IllegalConstruct(String::from("Exactly one argument expected")),
                        WSpan::from_syn(&punctuated_inside_expr),
                    ));
                }
                let mut expr = punctuated_inside_expr.into_iter().next().unwrap();
                self.visit_expr_mut(&mut expr);

                ExpandedSubproperty::Next(ExpandedNext { universal, expr })
            } else {
                if punctuated_inside_expr.len() != 2 {
                    return Err(Error::new(
                        ErrorType::IllegalConstruct(String::from("Exactly two arguments expected")),
                        WSpan::from_syn(&punctuated_inside_expr),
                    ));
                }

                let mut variable = None;

                if let Expr::Path(expr_path) = &punctuated_inside_expr[0] {
                    if let Some(ident) = expr_path.path.get_ident() {
                        variable = Some(ident.clone());
                    }
                }

                let Some(variable) = variable else {
                    return Err(Error::new(
                        ErrorType::IllegalConstruct(String::from(
                            "The first argument should be an identifier",
                        )),
                        WSpan::from_syn(&punctuated_inside_expr),
                    ));
                };

                let mut expr = punctuated_inside_expr.into_iter().nth(1).unwrap();
                self.visit_expr_mut(&mut expr);
                ExpandedSubproperty::FixedPoint(ExpandedFixedPoint {
                    universal,
                    variable,
                    expr,
                })
            };

            let ident = Ident::new(
                &format!(
                    "__mck_subproperty_{}",
                    self.expander.expanded_subproperties.len()
                ),
                mac.path.span(),
            );
            self.expander.expanded_subproperties.push(subproperty);
            let mut expr = create_expr_ident(ident);

            self.expanded_some_macro = true;
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

    fn rewrite_ctl_uni(
        &mut self,
        universal: bool,
        global: bool,
        mut mac: Macro,
        sufficient: Expr,
    ) -> Macro {
        // the general form is [lfp/gfp] Z . sufficient [outer_operator] (permitting [inner_operator] [A/E]X(Z))
        // for R, gfp Z . sufficient && (permitting || [A/E]X(Z))
        // for U, lfp Z . sufficient || (permitting && [A/E]X(Z))

        // for G, gfp Z . sufficient && ([A/E]X(Z))
        // for F, lfp Z . sufficient || ([A/E]X(Z))

        let span = mac.span();

        // choose greatest fixed points for global CTL properties

        // process the expr
        let variable = Ident::new("__mck_Z", span);

        let outer_operator = if global {
            BinOp::BitAnd(Token![&](span))
        } else {
            BinOp::BitOr(Token![|](span))
        };
        let next_path = if universal {
            path!(::machine_check::AX)
        } else {
            path!(::machine_check::EX)
        };

        let next_expr = Expr::Macro(ExprMacro {
            attrs: vec![],
            mac: Macro {
                path: next_path,
                bang_token: Token![!](span),
                delimiter: syn::MacroDelimiter::Paren(Paren::default()),
                tokens: variable.clone().into_token_stream(),
            },
        });

        let expr = Expr::Binary(ExprBinary {
            attrs: vec![],
            left: Box::new(sufficient),
            op: outer_operator,
            right: Box::new(next_expr),
        });

        let args: Punctuated<Expr, Token![,]> =
            Punctuated::from_iter([create_expr_ident(variable), expr]);

        mac.tokens = args.into_token_stream();

        let fixed_point = if global { "gfp" } else { "lfp" };
        let ident = &mut mac.path.segments[1].ident;
        *ident = Ident::new(fixed_point, ident.span());

        mac
    }
}
