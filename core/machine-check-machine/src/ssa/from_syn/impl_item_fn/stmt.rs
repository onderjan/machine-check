use std::collections::HashMap;

use syn::{punctuated::Punctuated, spanned::Spanned, Block, Expr, ExprAssign, Pat, Stmt, Token};

use crate::{
    ssa::{error::{DescriptionErrors}, from_syn::{impl_item_fn::FunctionScope, ty::fold_partial_general_type}}, util::{create_expr_ident, path_matches_global_names}, wir::{
        WBlock, WExprCall, WIdent, WIndexedExpr, WIndexedIdent, WMacroableCallFunc,
        WPanicMacroKind, WPartialGeneralType, WStmt, WStmtAssign, WStmtIf, ZTac,
    },
};

impl super::FunctionFolder {
    pub fn fold_block(
        &mut self,
        block: Block,
    ) -> Result<(WBlock<ZTac>, Option<WIdent>), DescriptionErrors> {
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

        let result_stmt = if let Some(Stmt::Expr(_, None)) = orig_stmts.last() {
            orig_stmts.pop()
        } else {
            None
        };

        let mut stmts: Vec<WStmt<ZTac>> = Vec::new();
        let mut errors = Vec::new();

        for orig_stmt in orig_stmts {
            match self.fold_stmt(scope_id, orig_stmt, &mut stmts) {
                Ok(()) => {},
                Err(err) => errors.push(err),
            }
        }

        let mut pre_return_stmts = Vec::new();
        let return_ident =
            // has a return statement
            if let Some(result_stmt) = result_stmt {
                let Stmt::Expr(expr, None) = result_stmt else {
                    panic!(
                        "Result statement should be an expression: {:?}",
                        result_stmt
                    );
                };
                match self.force_right_expr_to_ident(expr, &mut pre_return_stmts) {
                    Ok(ident) => Some(ident),
                    Err(err) => {
                        errors.push(DescriptionErrors::single(err));
                        // the None value will never propagate out of the function
                        None
                    },
                } 
        } else {
            None
        };

        DescriptionErrors::errors_vec_to_result(errors)?;

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
    ) -> Result<(), DescriptionErrors> {
        match stmt {
            Stmt::Local(local) => {
                let mut pat = local.pat.clone();
                let mut ty = WPartialGeneralType::Unknown;
                if let Pat::Type(pat_type) = pat {
                    ty = fold_partial_general_type(*pat_type.ty)?;
                    pat = *pat_type.pat;
                }

                let Pat::Ident(left_pat_ident) = pat else {
                    // TODO: this should be an error
                    panic!("Local pattern should be an ident: {:?}", pat)
                };
                let local_syn_ident = left_pat_ident.ident;
                let local_ident = WIdent::from_syn_ident(local_syn_ident.clone());
                self.add_local_ident(scope_id, local_ident, ty);

                if let Some(init) = local.init {
                    if init.diverge.is_some() {
                        panic!("Diverging let not supported");
                    }
                    self.fold_stmt_expr(Expr::Assign(ExprAssign { 
                        attrs: vec![], 
                        left: Box::new(create_expr_ident(local_syn_ident)),
                        eq_token: init.eq_token,
                        right: init.expr
                    }), 
                    result_stmts)?;
                }
            }
            Stmt::Expr(stmt_expr, _) => {
                self.fold_stmt_expr(stmt_expr, result_stmts)?
            }
            _ => panic!("Unexpected type of statement: {:?}", stmt),
        };
        Ok(())
    }

    fn fold_stmt_expr(
        &mut self,
        stmt_expr: Expr,
        result_stmts: &mut Vec<WStmt<ZTac>>,
    ) -> Result<(), DescriptionErrors> {
        match stmt_expr {
            syn::Expr::Assign(expr) => {
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
                    _ => panic!("Left expr should be ident or index"),
                };

                let right = self.fold_right_expr(*expr.right, result_stmts)?;

                result_stmts.push(WStmt::Assign(WStmtAssign { left, right }));
            }
            syn::Expr::If(expr_if) => {
                let condition = self.force_right_expr_to_call_arg(*expr_if.cond, result_stmts)?;
                let then_block = self.fold_block(expr_if.then_branch)?.0;

                let mut else_stmts = Vec::new();
                if let Some((_else_token, else_branch)) = expr_if.else_branch {
                    self.fold_stmt_expr(*else_branch, &mut else_stmts)?;
                }
                let else_block = WBlock { stmts: else_stmts };

                result_stmts.push(WStmt::If(WStmtIf {
                    condition,
                    then_block,
                    else_block,
                }));
            }
            syn::Expr::Block(expr_block) => {
                // handle nested blocks
                let (mut block, result) = self.fold_block(expr_block.block)?;
                assert!(result.is_none());
                result_stmts.append(&mut block.stmts);
            }
            syn::Expr::Macro(expr_macro) => {
                let span = expr_macro.span();
                let mac = expr_macro.mac;
                let kind = if path_matches_global_names(&mac.path, &["std", "panic"]) {
                    Some(WPanicMacroKind::Panic)
                } else if path_matches_global_names(&mac.path, &["std", "unimplemented"]) {
                    Some(WPanicMacroKind::Unimplemented)
                } else if path_matches_global_names(&mac.path, &["std", "todo"]) {
                    Some(WPanicMacroKind::Todo)
                } else {
                    None
                };
                let args =
                    match mac.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated) {
                        Ok(args) => args,
                        Err(_) => panic!("Could not parse macro args"),
                    };

                let mut call_args = Vec::new();
                for arg in args {
                    let Expr::Lit(lit) = arg else {
                        panic!("Unexpected non-literal arg");
                    };
                    call_args.push(crate::wir::WCallArg::Literal(lit.lit));
                }

                let Some(kind) = kind else {
                    panic!("Unsupported macro");
                };
                result_stmts.push(WStmt::Assign(WStmtAssign {
                    left: WIndexedIdent::NonIndexed(WIdent::new(String::from("__mck_x"), span)),
                    right: WIndexedExpr::NonIndexed(crate::wir::WExpr::Call(WExprCall {
                        fn_path: WMacroableCallFunc::PanicMacro(kind),
                        args: call_args,
                    })),
                }));
            }
            _ => panic!("Unexpected type of expression: {:?}", stmt_expr),
        };
        Ok(())
    }
}
