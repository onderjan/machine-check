use syn::{
    parse::Parser,
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    visit_mut::{self, VisitMut},
    Expr, Ident, Item, Lit, Macro, Path, PathArguments, PathSegment, Stmt, Token,
};

use crate::{
    support::types::machine_check_bitvector_new,
    util::{create_expr_call, create_expr_path, path_matches_global_names, ArgType},
    ErrorType, MachineError,
};

pub fn expand_macros(items: &mut [Item]) -> Result<(), MachineError> {
    let mut visitor = Visitor {
        result: Ok(()),
        panic_messages: Vec::new(),
    };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }

    visitor.result
}

struct Visitor {
    result: Result<(), MachineError>,
    panic_messages: Vec<String>,
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
    fn process_macro(&mut self, mac: &mut Macro) -> Result<Expr, MachineError> {
        if path_matches_global_names(&mac.path, &["machine_check", "bitmask_switch"]) {
            return self.process_bitmask_switch(mac);
        }
        if path_matches_global_names(&mac.path, &["std", "panic"]) {
            return self.process_panic(mac, None);
        }
        if path_matches_global_names(&mac.path, &["std", "unimplemented"]) {
            return self.process_panic(mac, Some("not implemented"));
        }
        if path_matches_global_names(&mac.path, &["std", "todo"]) {
            return self.process_panic(mac, Some("not yet implemented"));
        }
        Err(MachineError::new(ErrorType::UnknownMacro, mac.span()))
    }

    fn process_bitmask_switch(&self, mac: &mut Macro) -> Result<Expr, MachineError> {
        let span = mac.span();
        let macro_result =
            match machine_check_bitmask_switch::process(::std::mem::take(&mut mac.tokens)) {
                Ok(ok) => ok,
                Err(err) => {
                    return Err(MachineError::new(ErrorType::MacroError(err.msg()), span));
                }
            };
        parse2(macro_result).map_err(|err| MachineError::new(ErrorType::MacroParseError(err), span))
    }

    fn process_panic(
        &mut self,
        mac: &mut Macro,
        prefix: Option<&str>,
    ) -> Result<Expr, MachineError> {
        let tokens_span = mac.tokens.span();
        // no formatting supported, only a single string
        let arguments = match Punctuated::<Expr, Comma>::parse_terminated
            .parse2(::std::mem::take(&mut mac.tokens))
        {
            Ok(ok) => ok,
            Err(err) => {
                return Err(MachineError::new(
                    ErrorType::MacroParseError(err),
                    tokens_span,
                ))
            }
        };

        let message = match arguments.len() {
            0 => String::from(prefix.unwrap_or("explicit panic")),
            1 => {
                let first_arg = &arguments[0];
                let Expr::Lit(expr_lit) = first_arg else {
                    return Err(MachineError::new(
                        ErrorType::UnsupportedConstruct(String::from(
                            "Unsupported non-literal first panic-macro argument",
                        )),
                        first_arg.span(),
                    ));
                };
                let Lit::Str(lit_str) = &expr_lit.lit else {
                    return Err(MachineError::new(
                        ErrorType::UnsupportedConstruct(String::from(
                            "Unsupported non-string-literal first panic-macro argument",
                        )),
                        first_arg.span(),
                    ));
                };
                let format_string = lit_str.value();
                let format_string_without_escapes =
                    format_string.replace("{{", "").replace("}}", "");
                if format_string_without_escapes.contains('{')
                    || format_string_without_escapes.contains('}')
                {
                    return Err(MachineError::new(
                        ErrorType::UnsupportedConstruct(String::from(
                            "Unsupported panic-macro formatting parameters",
                        )),
                        tokens_span,
                    ));
                }

                if let Some(prefix) = prefix {
                    format!("{}: {}", prefix, format_string)
                } else {
                    format_string
                }
            }
            _ => {
                return Err(MachineError::new(
                    ErrorType::UnsupportedConstruct(String::from(
                        "Unsupported panic-macro parameters after formatting string",
                    )),
                    tokens_span,
                ))
            }
        };

        // add the message to panic messages
        self.panic_messages.push(message);
        let message_index_plus_one = self.panic_messages.len();

        let Ok(message_index_plus_one): Result<u32, _> = message_index_plus_one.try_into() else {
            return Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from("Too many panic calls")),
                mac.span(),
            ));
        };
        // convert to panic call
        let path = Path {
            leading_colon: Some(Token![::](mac.span())),
            segments: Punctuated::<PathSegment, Token![::]>::from_iter([
                PathSegment {
                    ident: Ident::new("machine_check", mac.span()),
                    arguments: PathArguments::None,
                },
                PathSegment {
                    ident: Ident::new("internal", mac.span()),
                    arguments: PathArguments::None,
                },
                PathSegment {
                    ident: Ident::new("panic", mac.span()),
                    arguments: PathArguments::None,
                },
            ]),
        };
        Ok(create_expr_call(
            create_expr_path(path),
            vec![(
                ArgType::Normal,
                machine_check_bitvector_new(32, &message_index_plus_one.to_string()),
            )],
        ))
    }
}
