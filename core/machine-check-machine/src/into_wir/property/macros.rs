use quote::ToTokens;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::Paren,
    visit_mut::{self, VisitMut},
    Attribute, BinOp, Expr, ExprBinary, ExprMacro, Ident, Macro, Stmt, Token,
};
use syn_path::path;

use crate::{
    into_wir::{
        property::{ExprProperty, ExprSubproperty},
        Error, ErrorType,
    },
    util::{create_expr_ident, path_matches_global_names},
    wir::{WFixedPointOperator, WIdent, WNextOperator, WSpan},
};

pub fn expand_property_macros(property: &mut ExprProperty) -> Result<bool, Error> {
    println!("Expanding property macros");
    let mut visitor = Visitor {
        num_subproperties: property.subproperties.len(),
        current_subproperty: 0,
        result: Ok(()),
        expanded_some_macro: false,
        new_subproperties: Vec::new(),
    };
    for (index, subproperty) in property.subproperties.iter_mut().enumerate() {
        println!("Visiting subproperty {}", index);
        visitor.current_subproperty = index;
        if let ExprSubproperty::Expr(expr, _children) = subproperty {
            visitor.visit_expr_mut(expr);
        }
    }

    for (parent_index, new_subproperty) in visitor.new_subproperties {
        if let Some(parent_index) = parent_index {
            let new_subproperty_index = property.subproperties.len();

            let parent = &mut property.subproperties[parent_index];
            if let ExprSubproperty::Expr(_expr, children) = parent {
                children.push(new_subproperty_index);
            }
        }
        property.subproperties.push(new_subproperty);
    }

    visitor.result?;
    Ok(visitor.expanded_some_macro)
}

struct Visitor {
    num_subproperties: usize,
    current_subproperty: usize,
    result: Result<(), Error>,
    expanded_some_macro: bool,
    new_subproperties: Vec<(Option<usize>, ExprSubproperty)>,
}

impl VisitMut for Visitor {
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        if let Stmt::Macro(stmt_macro) = stmt {
            // process macro
            match self.process_macro(stmt_macro.mac.clone(), stmt_macro.attrs.clone()) {
                Ok(macro_result) => *stmt = Stmt::Expr(macro_result, stmt_macro.semi_token),
                Err(err) => self.push_error(err),
            }
        } else {
            // delegate
            visit_mut::visit_stmt_mut(self, stmt);
        }
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Expr::Macro(expr_macro) = expr {
            // process macro
            match self.process_macro(expr_macro.mac.clone(), expr_macro.attrs.clone()) {
                Ok(macro_result) => *expr = macro_result,
                Err(err) => self.push_error(err),
            }
        } else {
            // delegate
            visit_mut::visit_expr_mut(self, expr);
        }
    }
}

impl Visitor {
    fn process_macro(&mut self, mac: Macro, attrs: Vec<Attribute>) -> Result<Expr, Error> {
        println!(
            "Visiting macro {}",
            quote::ToTokens::into_token_stream(mac.clone().into_token_stream())
        );

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

            let outer_subproperty_index = self.num_subproperties + self.new_subproperties.len();

            let inner_subproperty_index = outer_subproperty_index + 1;

            let (outer_subproperty, inner_expr) = if ex || ax {
                if punctuated_inside_expr.len() != 1 {
                    return Err(Error::new(
                        ErrorType::IllegalConstruct(String::from("Exactly one argument expected")),
                        WSpan::from_syn(&punctuated_inside_expr),
                    ));
                }
                let expr = punctuated_inside_expr.into_iter().next().unwrap();

                let outer_subproperty = ExprSubproperty::Next(WNextOperator {
                    universal,
                    inner: inner_subproperty_index,
                });

                (outer_subproperty, expr)
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
                        variable = Some(WIdent::from_syn_ident(ident.clone()));
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

                let expr = punctuated_inside_expr.into_iter().nth(1).unwrap();

                let outer_subproperty = ExprSubproperty::FixedPoint(WFixedPointOperator {
                    universal,
                    variable,
                    inner: inner_subproperty_index,
                });

                (outer_subproperty, expr)
            };

            let inner_subproperty = ExprSubproperty::Expr(inner_expr, Vec::new());

            let ident = Ident::new(
                &format!("__mck_subproperty_{}", outer_subproperty_index),
                mac.path.span(),
            );

            // TODO: update inner subproperties

            self.new_subproperties
                .push((Some(self.current_subproperty), outer_subproperty));
            self.new_subproperties.push((None, inner_subproperty));
            let expr = create_expr_ident(ident);

            self.expanded_some_macro = true;
            println!("Expanded mu-calculus macro");
            return Ok(expr);
        }
        println!("Not expanded");
        Ok(Expr::Macro(ExprMacro { attrs, mac }))
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

        println!("Rewritten: {}", quote::quote!(#mac));

        mac
    }
}
