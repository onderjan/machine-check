use machine_check_common::PropertyMacros;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Attribute, BinOp, Expr, ExprBinary, ExprMacro, Ident, Macro, Stmt, Token,
};

use crate::{
    into_wir::{
        property::{ExprProperty, ExprSubproperty, ExprSubpropertyFunc},
        Error, ErrorType,
    },
    util::{create_expr_ident, extract_path_ident, path_matches_global_names},
    wir::{WIdent, WSpan, WSubpropertyFixedPoint, WSubpropertyNext},
};

pub fn expand_property_macros<D>(
    property: &mut ExprProperty,
    property_macros: &PropertyMacros<D>,
) -> Result<bool, Error> {
    let mut visitor = Visitor {
        property_macros,
        num_subproperties: property.subproperties.len(),
        current_subproperty: 0,
        result: Ok(()),
        expanded_some_macro: false,
        new_subproperties: Vec::new(),
    };
    for (index, subproperty) in property.subproperties.iter_mut().enumerate() {
        visitor.current_subproperty = index;
        if let ExprSubproperty::Expr(subproperty_func) = subproperty {
            visitor.visit_expr_mut(&mut subproperty_func.expr);
        }
    }

    for mut new_subproperty in visitor.new_subproperties {
        if let Some(parent_index) = new_subproperty.parent() {
            let new_subproperty_index = property.subproperties.len();

            let parent = &mut property.subproperties[parent_index];
            if let ExprSubproperty::Expr(parent) = parent {
                parent.dependencies.push(new_subproperty_index);

                if parent.display.is_some() && is_trivial_parent(&parent.expr) {
                    // do not display the information in child
                    match &mut new_subproperty {
                        ExprSubproperty::Expr(new_subproperty) => new_subproperty.display = None,
                        ExprSubproperty::Next(new_subproperty) => new_subproperty.display = None,
                        ExprSubproperty::FixedPoint(new_subproperty) => {
                            new_subproperty.display = None;
                        }
                    }
                }
            }
        }
        property.subproperties.push(new_subproperty);
    }

    visitor.result?;
    Ok(visitor.expanded_some_macro)
}

fn is_trivial_parent(expr: &Expr) -> bool {
    let Expr::Path(expr) = expr else {
        return false;
    };
    let Some(ident) = extract_path_ident(&expr.path) else {
        return false;
    };
    ident.to_string().starts_with("__mck_subproperty_")
}

struct Visitor<'a, D> {
    property_macros: &'a PropertyMacros<D>,
    num_subproperties: usize,
    current_subproperty: usize,
    result: Result<(), Error>,
    expanded_some_macro: bool,
    new_subproperties: Vec<ExprSubproperty>,
}

impl<D> VisitMut for Visitor<'_, D> {
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

impl<D> Visitor<'_, D> {
    fn process_macro(&mut self, mac: Macro, attrs: Vec<Attribute>) -> Result<Expr, Error> {
        if let Some(macro_ident) = mac.path.get_ident() {
            if let Some(property_macro) = self.property_macros.macros.get(&macro_ident.to_string())
            {
                // expand property macro
                let tokens_span = mac.tokens.span();
                let expanded =
                    (property_macro)(&self.property_macros.data, mac.tokens).map_err(|err| {
                        Error::new(
                            ErrorType::MacroParseError(syn::Error::new(tokens_span, err)),
                            WSpan::from_span(tokens_span),
                        )
                    })?;

                let parsed_expr = syn::parse2(expanded).map_err(|err| {
                    let span = err.span();
                    Error::new(ErrorType::MacroParseError(err), WSpan::from_span(span))
                })?;
                return Ok(parsed_expr);
            }
        }

        let ef = path_matches_global_names(&mac.path, &["machine_check", "EF"]);
        let af = path_matches_global_names(&mac.path, &["machine_check", "AF"]);
        let eg = path_matches_global_names(&mac.path, &["machine_check", "EG"]);
        let ag = path_matches_global_names(&mac.path, &["machine_check", "AG"]);

        let eu = path_matches_global_names(&mac.path, &["machine_check", "EU"]);
        let au = path_matches_global_names(&mac.path, &["machine_check", "AU"]);
        let er = path_matches_global_names(&mac.path, &["machine_check", "ER"]);
        let ar = path_matches_global_names(&mac.path, &["machine_check", "AR"]);

        let uni_argument_ctl = ef || af || eg || ag;
        let bi_argument_ctl = eu || au || er || ar;

        let prev_expanded_macro = self.expanded_some_macro;
        self.expanded_some_macro = true;

        if uni_argument_ctl || bi_argument_ctl {
            let universal = af || ag || au || ar;
            let greatest = eg || ag || er || ar;

            return self.expand_ctl(universal, greatest, bi_argument_ctl, mac);
        }

        let ex = path_matches_global_names(&mac.path, &["machine_check", "EX"]);
        let ax = path_matches_global_names(&mac.path, &["machine_check", "AX"]);

        if ex || ax {
            return self.expand_next(ax, mac);
        }

        let lfp = path_matches_global_names(&mac.path, &["machine_check", "lfp"]);
        let gfp = path_matches_global_names(&mac.path, &["machine_check", "gfp"]);

        if lfp || gfp {
            return self.expand_fixed_point(gfp, mac);
        }

        self.expanded_some_macro = prev_expanded_macro;

        Ok(Expr::Macro(ExprMacro { attrs, mac }))
    }

    fn push_error(&mut self, err: Error) {
        if self.result.is_ok() {
            self.result = Err(err);
        }
    }

    fn expand_next(&mut self, universal: bool, mac: Macro) -> Result<Expr, Error> {
        let span = mac.span();
        let inner_expr = parse_uni_argument_macro(&mac)?;
        let outer_display = make_machine_check_macro_display(&mac);

        let outer_subproperty_index = self.num_subproperties + self.new_subproperties.len();
        let inner_subproperty_index = outer_subproperty_index + 1;

        let outer_subproperty = ExprSubproperty::Next(WSubpropertyNext {
            parent: Some(self.current_subproperty),
            universal,
            inner: inner_subproperty_index,
            display: Some(outer_display),
        });

        let inner_subproperty = ExprSubproperty::Expr(ExprSubpropertyFunc {
            parent: Some(outer_subproperty_index),
            expr: inner_expr,
            dependencies: Vec::new(),
            display: None,
        });

        self.new_subproperties.push(outer_subproperty);
        self.new_subproperties.push(inner_subproperty);
        let expr = create_expr_ident(subproperty_ident(outer_subproperty_index, span));
        Ok(expr)
    }

    fn expand_fixed_point(&mut self, universal: bool, mac: Macro) -> Result<Expr, Error> {
        let span = mac.span();
        let (variable, inner_expr) = parse_bi_argument_macro(&mac)?;
        let outer_display = make_machine_check_macro_display(&mac);

        let Expr::Path(variable) = variable else {
            return Err(Error::new(
                ErrorType::IllegalConstruct(String::from(
                    "The first argument (fixed-point variable) should be an identifier",
                )),
                WSpan::from_syn(&variable),
            ));
        };
        let Some(variable) = extract_path_ident(&variable.path) else {
            return Err(Error::new(
                ErrorType::IllegalConstruct(String::from(
                    "The first argument (fixed-point variable) should be an identifier",
                )),
                WSpan::from_syn(&variable),
            ));
        };
        let variable = WIdent::from_syn_ident(variable.clone());

        let outer_subproperty_index = self.num_subproperties + self.new_subproperties.len();
        let inner_subproperty_index = outer_subproperty_index + 1;

        let outer_subproperty = ExprSubproperty::FixedPoint(WSubpropertyFixedPoint {
            parent: Some(self.current_subproperty),
            greatest: universal,
            variable,
            inner: inner_subproperty_index,
            display: Some(outer_display),
        });

        let inner_subproperty = ExprSubproperty::Expr(ExprSubpropertyFunc {
            parent: Some(outer_subproperty_index),
            expr: inner_expr,
            dependencies: Vec::new(),
            display: None,
        });

        self.new_subproperties.push(outer_subproperty);
        self.new_subproperties.push(inner_subproperty);

        let expr = create_expr_ident(subproperty_ident(outer_subproperty_index, span));

        Ok(expr)
    }

    fn expand_ctl(
        &mut self,
        universal: bool,
        greatest: bool,
        bi_argument: bool,
        mac: Macro,
    ) -> Result<Expr, Error> {
        let fixed_point_display = make_machine_check_macro_display(&mac);
        let span = mac.span();
        let (permitting, sufficient) = if bi_argument {
            let (permitting, sufficient) = parse_bi_argument_macro(&mac)?;
            (Some(permitting), sufficient)
        } else {
            (None, parse_uni_argument_macro(&mac)?)
        };

        // the general form is [lfp/gfp] Z . sufficient [outer_operator] (permitting [inner_operator] [A/E]X(Z))
        // for R, gfp Z . sufficient && (permitting || [A/E]X(Z))
        // for U, lfp Z . sufficient || (permitting && [A/E]X(Z))

        // for G, gfp Z . sufficient && ([A/E]X(Z))
        // for F, lfp Z . sufficient || ([A/E]X(Z))

        // create the fixed point

        let fixed_point_index = self.num_subproperties + self.new_subproperties.len();
        let outer_func_index = fixed_point_index + 1;
        let (permitting_index, sufficient_index) = if permitting.is_some() {
            (Some(outer_func_index + 1), outer_func_index + 2)
        } else {
            (None, outer_func_index + 1)
        };
        let next_operator_index = sufficient_index + 1;
        let next_func_index = next_operator_index + 1;

        let fixed_point_variable = WIdent::new(String::from("__mck_Z"), span);

        let fixed_point = ExprSubproperty::FixedPoint(WSubpropertyFixedPoint {
            parent: Some(self.current_subproperty),
            greatest,
            variable: fixed_point_variable.clone(),
            inner: outer_func_index,
            display: Some(fixed_point_display),
        });

        // create the outer function, it will not be displayed

        fn logical_bi_operator(is_and: bool, span: Span) -> BinOp {
            if is_and {
                BinOp::BitAnd(Token![&](span))
            } else {
                BinOp::BitOr(Token![|](span))
            }
        }

        let next_operator_expr = create_expr_ident(subproperty_ident(next_operator_index, span));

        let inner_expr = if let Some(permitting_index) = permitting_index {
            // the operator must be dual here, negate greatest
            let inner_operator = logical_bi_operator(!greatest, span);
            let permitting_ident = subproperty_ident(permitting_index, span);
            Expr::Binary(ExprBinary {
                attrs: vec![],
                left: Box::new(create_expr_ident(permitting_ident)),
                op: inner_operator,
                right: Box::new(next_operator_expr),
            })
        } else {
            next_operator_expr
        };

        let sufficient_ident = subproperty_ident(sufficient_index, span);

        let outer_expr = Expr::Binary(ExprBinary {
            attrs: vec![],
            left: Box::new(create_expr_ident(sufficient_ident)),
            op: logical_bi_operator(greatest, span),
            right: Box::new(inner_expr),
        });

        let outer = ExprSubproperty::Expr(ExprSubpropertyFunc {
            parent: Some(fixed_point_index),
            expr: outer_expr,
            dependencies: Vec::new(),
            display: None,
        });

        // create new subproperties for permitting and sufficient so they can be inspected

        let permitting = permitting.map(|permitting| {
            let display = make_expr_display(&permitting);
            ExprSubproperty::Expr(ExprSubpropertyFunc {
                parent: Some(outer_func_index),
                expr: permitting,
                dependencies: Vec::new(),
                display: Some(display),
            })
        });

        let sufficient_display = make_expr_display(&sufficient);
        let sufficient = ExprSubproperty::Expr(ExprSubpropertyFunc {
            parent: Some(outer_func_index),
            expr: sufficient,
            dependencies: Vec::new(),
            display: Some(sufficient_display),
        });

        // create the next operator and function, they will not be displayed

        let next_operator = ExprSubproperty::Next(WSubpropertyNext {
            parent: Some(outer_func_index),
            universal,
            inner: next_func_index,
            display: None,
        });

        let next_func = ExprSubproperty::Expr(ExprSubpropertyFunc {
            parent: Some(next_operator_index),
            expr: create_expr_ident(subproperty_ident(fixed_point_index, span)),
            dependencies: Vec::new(),
            display: None,
        });

        // add all to the vector

        self.new_subproperties.push(fixed_point);
        self.new_subproperties.push(outer);

        if let Some(permitting) = permitting {
            self.new_subproperties.push(permitting);
        }
        self.new_subproperties.push(sufficient);

        self.new_subproperties.push(next_operator);
        self.new_subproperties.push(next_func);

        let expr = create_expr_ident(subproperty_ident(fixed_point_index, span));

        Ok(expr)
    }
}

fn parse_uni_argument_macro(mac: &Macro) -> Result<Expr, Error> {
    let punctuated_inside_expr = parse_punctuated_in_macro(mac)?;
    if punctuated_inside_expr.len() != 1 {
        return Err(Error::new(
            ErrorType::IllegalConstruct(String::from("Exactly one argument to macro expected")),
            WSpan::from_syn(&punctuated_inside_expr),
        ));
    }
    let mut iter = punctuated_inside_expr.into_iter();
    let first_arg = iter.next().unwrap();
    Ok(first_arg)
}

fn parse_bi_argument_macro(mac: &Macro) -> Result<(Expr, Expr), Error> {
    let punctuated_inside_expr = parse_punctuated_in_macro(mac)?;
    if punctuated_inside_expr.len() != 2 {
        return Err(Error::new(
            ErrorType::IllegalConstruct(String::from("Exactly two argument to macro expected")),
            WSpan::from_syn(&punctuated_inside_expr),
        ));
    }
    let mut iter = punctuated_inside_expr.into_iter();
    let first_arg = iter.next().unwrap();
    let second_arg = iter.next().unwrap();
    Ok((first_arg, second_arg))
}

fn parse_punctuated_in_macro(mac: &Macro) -> Result<Punctuated<Expr, Token![,]>, Error> {
    mac.parse_body_with(Punctuated::parse_terminated)
        .map_err(|err| {
            let err_span = err.span();
            Error::new(ErrorType::MacroParseError(err), WSpan::from_span(err_span))
        })
}

fn subproperty_ident(index: usize, span: Span) -> Ident {
    Ident::new(&format!("__mck_subproperty_{}", index), span)
}

fn make_machine_check_macro_display(mac: &Macro) -> String {
    format!(
        "{}![{}]",
        mac.path.segments[1].ident,
        mac.tokens.to_token_stream()
    )
}

fn make_expr_display(expr: &Expr) -> String {
    expr.to_token_stream().to_string()
}
