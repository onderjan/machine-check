use anyhow::anyhow;
use proc_macro2::Span;
use quote::quote;
use syn::{visit_mut::VisitMut, Block, Expr, Ident, Pat, Stmt};

use crate::machine::util::{create_expr_path, create_let_stmt_from_pat_expr, create_pat_ident};

pub fn apply(file: &mut syn::File) -> anyhow::Result<()> {
    // apply linear SSA to each block using a visitor
    struct Visitor(anyhow::Result<()>);
    impl VisitMut for Visitor {
        fn visit_block_mut(&mut self, block: &mut Block) {
            let result = apply_to_block(block);
            if self.0.is_ok() {
                self.0 = result;
            }
            // do not delegate, the representation should not contain
            // any nested blocks anyway after translation
        }
    }
    let mut visitor = Visitor(Ok(()));
    visitor.visit_file_mut(file);
    visitor.0
}

fn apply_to_block(block: &mut Block) -> anyhow::Result<()> {
    let mut translator = BlockTranslator {
        translated_stmts: Vec::new(),
    };
    // apply linear SSA to statements one by one
    for stmt in &block.stmts {
        if let Err(err) = translator.apply_to_stmt(stmt.clone()) {
            return Err(err.context(format!(
                "Error transcribing statement to SSA: {}",
                quote!(#stmt)
            )));
        }
    }
    block.stmts = translator.translated_stmts;
    Ok(())
}
struct BlockTranslator {
    translated_stmts: Vec<Stmt>,
}

impl BlockTranslator {
    fn apply_to_stmt(&mut self, mut stmt: Stmt) -> anyhow::Result<()> {
        match stmt {
            Stmt::Expr(ref mut expr, _) => {
                // apply translation to expression without forced movement
                self.apply_to_expr(expr)?;
            }
            Stmt::Local(ref mut local) => {
                let Pat::Ident(ident) = &local.pat else {
                    return Err(anyhow!(
                        "Local let with non-ident pattern not supported"
                    ));
                };
                if ident.by_ref.is_some() || ident.mutability.is_some() || ident.subpat.is_some() {
                    return Err(anyhow!("Non-bare local let ident not supported"));
                }

                if let Some(ref mut init) = local.init {
                    if init.diverge.is_some() {
                        return Err(anyhow!("Local let with diverging else not supported"));
                    }

                    // apply translation to expression without forced movement
                    self.apply_to_expr(init.expr.as_mut())?;
                }
            }
            _ => return Err(anyhow!("Statement type {:?} not supported", stmt)),
        }
        self.translated_stmts.push(stmt);
        Ok(())
    }

    fn apply_to_expr(&mut self, expr: &mut Expr) -> anyhow::Result<()> {
        match expr {
            syn::Expr::Path(_) | syn::Expr::Lit(_) => {
                // do nothing, paths and literals are not moved in our SSA
            }
            syn::Expr::Field(field) => {
                // move base
                self.move_through_temp(&mut field.base)?;
            }
            syn::Expr::Paren(paren) => {
                // move statement in parentheses
                self.move_through_temp(&mut paren.expr)?;
                // remove parentheses
                *expr = (*paren.expr).clone();
            }
            syn::Expr::Call(call) => {
                // move call function expression and arguments
                self.move_through_temp(&mut call.func)?;
                for arg in &mut call.args {
                    self.move_through_temp(arg)?;
                }
            }
            syn::Expr::Struct(expr_struct) => {
                // move field values
                for field in &mut expr_struct.fields {
                    self.move_through_temp(&mut field.expr)?;
                }
                if expr_struct.rest.is_some() {
                    return Err(anyhow!("Struct rest not supported"));
                }
            }
            _ => {
                return Err(anyhow!("Expression type {:?} not supported", expr));
            }
        }
        Ok(())
    }

    fn move_through_temp(&mut self, expr: &mut Expr) -> anyhow::Result<()> {
        match expr {
            syn::Expr::Path(_) | syn::Expr::Lit(_) => {
                // do nothing, paths and literals are not moved in our SSA
                return Ok(());
            }
            syn::Expr::Paren(paren) => {
                // move statement in parentheses
                self.move_through_temp(&mut paren.expr)?;
                // remove parentheses
                *expr = (*paren.expr).clone();
                return Ok(());
            }
            _ => (),
        }

        // apply translation to expression
        // so that nested expressions are properly converted to SSA
        self.apply_to_expr(expr)?;

        // create a temporary variable
        let tmp_ident = Ident::new(
            format!("__mck_tmp_{}", self.translated_stmts.len()).as_str(),
            Span::call_site(),
        );

        // add new let statement to translated statements
        self.translated_stmts.push(create_let_stmt_from_pat_expr(
            Pat::Ident(create_pat_ident(tmp_ident.clone())),
            expr.clone(),
        ));

        // change expr to the temporary variable path
        *expr = create_expr_path(tmp_ident.into());
        Ok(())
    }
}
