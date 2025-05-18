use std::collections::HashMap;

use syn::{
    punctuated::Punctuated, spanned::Spanned, Block, Expr, ExprAssign, ExprIf, ExprMacro, Pat,
    Stmt, Token,
};

use crate::{
    description::{
        error::{Error, DescriptionErrorType, Errors},
        from_syn::{impl_item_fn::FunctionScope, ty::fold_type},
    },
    util::{create_expr_ident, path_matches_global_names},
    wir::{
        WBlock, WExprCall, WIdent, WIndexedExpr, WIndexedIdent, WMacroableCallFunc,
        WPanicMacroKind, WPartialGeneralType, WStmt, WStmtAssign, WStmtIf, ZTac,
    },
};

impl super::FunctionFolder {
    pub fn fold_block(
        &mut self,
        block: Block,
    ) -> Result<(WBlock<ZTac>, Option<WIdent>), Errors> {
        // push a local scope
        let scope_id = self.next_scope_id;
        self.next_scope_id = self
            .next_scope_id
            .checked_add(1)
            .expect("Scope id should not overflow");
        self.scopes.push(FunctionScope {
            local_map: HashMap::new(),
        });

        let mut orig_stmts = block.stmts;

        let result_expr = if let Some(Stmt::Expr(_, None)) = orig_stmts.last() {
            let result_stmt = orig_stmts.pop();
            let Some(Stmt::Expr(result_expr, None)) = result_stmt else {
                // this has been confirmed previously
                panic!("Last statement should be result expression");
            };
            Some(result_expr)
        } else {
            None
        };

        let mut stmts: Vec<WStmt<ZTac>> = Vec::new();
        let mut errors = Vec::new();

        for orig_stmt in orig_stmts {
            match self.fold_stmt(scope_id, orig_stmt, &mut stmts) {
                Ok(()) => {}
                Err(err) => errors.push(err),
            }
        }

        let mut pre_return_stmts = Vec::new();
        let return_ident =
            // has a return statement
            if let Some(result_expr) = result_expr {
                match self.force_right_expr_to_ident(result_expr, &mut pre_return_stmts) {
                    Ok(ident) => Some(ident),
                    Err(err) => {
                        errors.push(Errors::single(err));
                        // the None value will never propagate out of the function
                        None
                    },
                }
            } else {
                None
            };

        Errors::errors_vec_to_result(errors)?;

        stmts.extend(pre_return_stmts);

        // pop the local scope, it should exist
        assert!(self.scopes.pop().is_some());

        Ok((WBlock { stmts }, return_ident))
    }

    fn fold_stmt(
        &mut self,
        scope_id: u32,
        stmt: Stmt,
        result_stmts: &mut Vec<WStmt<ZTac>>,
    ) -> Result<(), Errors> {
        let stmt_span = stmt.span();
        match stmt {
            Stmt::Local(local) => {
                let mut pat = local.pat.clone();
                let mut ty = WPartialGeneralType::Unknown;
                if let Pat::Type(pat_type) = pat {
                    ty = fold_type(*pat_type.ty)
                        .map(WPartialGeneralType::Normal)
                        .map_err(Errors::single)?;
                    pat = *pat_type.pat;
                }

                let Pat::Ident(left_pat_ident) = pat else {
                    return Err(Errors::single(
                        Error::unsupported_construct(
                            "Non-ident local pattern",
                            pat.span(),
                        ),
                    ));
                };
                if left_pat_ident.by_ref.is_some() {
                    return Err(Errors::single(
                        Error::unsupported_construct(
                            "Pattern binding by reference",
                            left_pat_ident.by_ref.span(),
                        ),
                    ));
                }
                // mutable patterns are supported
                if let Some((_at, subpat)) = &left_pat_ident.subpat {
                    return Err(Errors::single(
                        Error::unsupported_construct("Subpatterns", subpat.span()),
                    ));
                }
                let local_syn_ident = left_pat_ident.ident;
                let local_ident = WIdent::from_syn_ident(local_syn_ident.clone());
                self.add_local_ident(scope_id, local_ident, ty);

                if let Some(init) = local.init {
                    if let Some((else_token, _else_block)) = init.diverge {
                        return Err(Errors::single(
                            Error::unsupported_construct(
                                "Diverging let",
                                else_token.span(),
                            ),
                        ));
                    }
                    self.fold_stmt_expr(
                        Expr::Assign(ExprAssign {
                            attrs: vec![],
                            left: Box::new(create_expr_ident(local_syn_ident)),
                            eq_token: init.eq_token,
                            right: init.expr,
                        }),
                        result_stmts,
                    )?;
                }
            }
            Stmt::Expr(stmt_expr, _) => self.fold_stmt_expr(stmt_expr, result_stmts)?,
            Stmt::Item(_) => {
                return Err(Errors::single(
                    Error::unsupported_construct("Items inside function", stmt_span),
                ))
            }
            Stmt::Macro(_) => {
                return Err(Errors::single(
                    Error::unsupported_construct(
                        "Macro invocations in statement position",
                        stmt_span,
                    ),
                ))
            }
        };
        Ok(())
    }

    fn fold_stmt_expr(
        &mut self,
        expr: Expr,
        result_stmts: &mut Vec<WStmt<ZTac>>,
    ) -> Result<(), Errors> {
        match expr {
            syn::Expr::Assign(expr) => self.fold_assign(expr, result_stmts),
            syn::Expr::If(expr) => self.fold_if(expr, result_stmts),
            syn::Expr::Block(expr) => {
                // handle nested blocks
                let (mut block, result) = self.fold_block(expr.block)?;
                if let Some(result) = result {
                    return Err(Errors::single(
                        Error::unsupported_construct(
                            "Block statements with result",
                            result.span(),
                        ),
                    ));
                };
                assert!(result.is_none());
                result_stmts.append(&mut block.stmts);
                Ok(())
            }
            syn::Expr::Macro(expr) => self.fold_macro(expr, result_stmts),
            _ => Err(Errors::single(
                Error::unsupported_construct("Expression kind", expr.span()),
            )),
        }
    }

    fn fold_assign(
        &mut self,
        expr: ExprAssign,
        result_stmts: &mut Vec<WStmt<ZTac>>,
    ) -> Result<(), Errors> {
        let left = match *expr.left {
            Expr::Index(expr_index) => {
                let base_ident = self.fold_expr_as_ident(*expr_index.expr)?;
                let index_ident =
                    self.force_right_expr_to_ident(*expr_index.index, result_stmts)?;
                WIndexedIdent::Indexed(base_ident, index_ident)
            }
            Expr::Path(expr_path) => {
                let left_ident = self.fold_expr_as_ident(Expr::Path(expr_path))?;

                WIndexedIdent::NonIndexed(left_ident.clone())
            }
            _ => {
                return Err(Errors::single(
                    Error::unsupported_construct(
                        "Left expression that is not an identifier nor index",
                        expr.span(),
                    ),
                ))
            }
        };

        let right = self.fold_right_expr(*expr.right, result_stmts)?;
        result_stmts.push(WStmt::Assign(WStmtAssign { left, right }));
        Ok(())
    }

    fn fold_if(
        &mut self,
        expr: ExprIf,
        result_stmts: &mut Vec<WStmt<ZTac>>,
    ) -> Result<(), Errors> {
        let condition = self.force_right_expr_to_call_arg(*expr.cond, result_stmts)?;
        let then_block = self.fold_block(expr.then_branch)?.0;

        let mut else_stmts = Vec::new();
        if let Some((_else_token, else_branch)) = expr.else_branch {
            self.fold_stmt_expr(*else_branch, &mut else_stmts)?;
        }
        let else_block = WBlock { stmts: else_stmts };

        result_stmts.push(WStmt::If(WStmtIf {
            condition,
            then_block,
            else_block,
        }));
        Ok(())
    }

    fn fold_macro(
        &mut self,
        expr: ExprMacro,
        result_stmts: &mut Vec<WStmt<ZTac>>,
    ) -> Result<(), Errors> {
        let macro_span = expr.span();
        let mac = expr.mac;
        let kind = if path_matches_global_names(&mac.path, &["std", "panic"]) {
            WPanicMacroKind::Panic
        } else if path_matches_global_names(&mac.path, &["std", "unimplemented"]) {
            WPanicMacroKind::Unimplemented
        } else if path_matches_global_names(&mac.path, &["std", "todo"]) {
            WPanicMacroKind::Todo
        } else {
            return Err(Errors::single(
                Error::unsupported_construct("This macro", mac.path.span()),
            ));
        };
        let args = match mac.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated) {
            Ok(args) => args,
            Err(err) => {
                return Err(Errors::single(Error::new(
                    DescriptionErrorType::MacroParseError(err),
                    mac.span(),
                )))
            }
        };

        let mut call_args = Vec::new();
        for arg in args {
            let Expr::Lit(lit) = arg else {
                return Err(Errors::single(
                    Error::unsupported_construct(
                        "Panic-like macro with a non-literal arg",
                        arg.span(),
                    ),
                ));
            };
            call_args.push(crate::wir::WCallArg::Literal(lit.lit));
        }

        result_stmts.push(WStmt::Assign(WStmtAssign {
            left: WIndexedIdent::NonIndexed(WIdent::new(String::from("__mck_x"), macro_span)),
            right: WIndexedExpr::NonIndexed(crate::wir::WExpr::Call(WExprCall {
                fn_path: WMacroableCallFunc::PanicMacro(kind),
                args: call_args,
            })),
        }));
        Ok(())
    }
}
