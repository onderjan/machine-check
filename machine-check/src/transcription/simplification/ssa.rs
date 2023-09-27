use anyhow::anyhow;
use proc_macro2::Span;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{
    visit_mut::VisitMut, Block, Expr, ExprPath, Ident, Local, LocalInit, Pat, PatIdent, Path,
    PathSegment, Stmt,
};

pub fn apply(file: &mut syn::File) -> anyhow::Result<()> {
    // apply transcription to operations to calls first
    super::ops_to_calls::apply(file);

    // apply transcription to each block using a visitor
    struct Visitor(anyhow::Result<()>);
    impl VisitMut for Visitor {
        fn visit_block_mut(&mut self, block: &mut Block) {
            let result = apply_to_block(block);
            if self.0.is_ok() {
                self.0 = result;
            }
            // do not delegate, the representation should not contain
            // any nested blocks anyway after transcription
        }
    }
    let mut visitor = Visitor(Ok(()));
    visitor.visit_file_mut(file);
    visitor.0
}

fn apply_to_block(block: &mut Block) -> anyhow::Result<()> {
    let mut transcriber = BlockTranscriber {
        transcribed_stmts: Vec::new(),
    };
    // apply transcription to statements one by one
    for stmt in &block.stmts {
        if let Err(err) = transcriber.apply_transcription_to_stmt(stmt.clone()) {
            return Err(err.context(format!(
                "Error transcribing statement to SSA: {}",
                quote!(#stmt)
            )));
        }
    }
    block.stmts = transcriber.transcribed_stmts;
    Ok(())
}
struct BlockTranscriber {
    transcribed_stmts: Vec<Stmt>,
}

impl BlockTranscriber {
    fn apply_transcription_to_stmt(&mut self, mut stmt: Stmt) -> anyhow::Result<()> {
        match stmt {
            Stmt::Expr(ref mut expr, semi) => {
                if semi.is_none() {
                    // force movement from return expression for ease of use
                    self.move_expression_through_temporary(expr)?;
                } else {
                    // apply transcription to expression without forced movement
                    self.apply_transcription_to_expression(expr)?;
                }
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

                    // apply transcription to expression without forced movement
                    self.apply_transcription_to_expression(init.expr.as_mut())?;
                }
            }
            _ => return Err(anyhow!("Statement type {:?} not supported", stmt)),
        }
        self.transcribed_stmts.push(stmt);
        Ok(())
    }

    fn apply_transcription_to_expression(&mut self, expr: &mut Expr) -> anyhow::Result<()> {
        match expr {
            syn::Expr::Path(_) | syn::Expr::Lit(_) => {
                // do nothing, paths and literals are not moved in our SSA
            }
            syn::Expr::Field(field) => {
                // move base
                self.move_expression_through_temporary(&mut field.base)?;
            }
            syn::Expr::Paren(paren) => {
                // move statement in parentheses
                self.move_expression_through_temporary(&mut paren.expr)?;
                // remove parentheses
                *expr = (*paren.expr).clone();
            }
            syn::Expr::Call(call) => {
                // move call function expression and arguments
                self.move_expression_through_temporary(&mut call.func)?;
                for arg in &mut call.args {
                    self.move_expression_through_temporary(arg)?;
                }
            }
            syn::Expr::Struct(expr_struct) => {
                // move field values and rest
                for field in &mut expr_struct.fields {
                    self.move_expression_through_temporary(&mut field.expr)?;
                }
                if let Some(ref mut rest) = expr_struct.rest {
                    self.move_expression_through_temporary(rest)?;
                }
            }
            _ => {
                return Err(anyhow!("Expression type {:?} not supported", expr));
            }
        }
        Ok(())
    }

    fn move_expression_through_temporary(&mut self, expr: &mut Expr) -> anyhow::Result<()> {
        match expr {
            syn::Expr::Path(_) | syn::Expr::Lit(_) => {
                // do nothing, paths and literals are not moved in our SSA
                return Ok(());
            }
            syn::Expr::Paren(paren) => {
                // move statement in parentheses
                self.move_expression_through_temporary(&mut paren.expr)?;
                // remove parentheses
                *expr = (*paren.expr).clone();
                return Ok(());
            }
            _ => (),
        }

        // apply transcription to expression
        // so that nested expressions are properly converted to SSA
        self.apply_transcription_to_expression(expr)?;

        // create a temporary variable
        let tmp_ident = Ident::new(
            format!("__mck_tmp_{}", self.transcribed_stmts.len()).as_str(),
            Span::call_site(),
        );

        // add new let statement to transcribed statements
        // i.e. let tmp = expr;
        let let_stmt = Stmt::Local(Local {
            attrs: vec![],
            let_token: Default::default(),
            pat: Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: tmp_ident.clone(),
                subpat: None,
            }),
            init: Some(LocalInit {
                eq_token: Default::default(),
                expr: Box::new(expr.clone()),
                diverge: None,
            }),
            semi_token: Default::default(),
        });
        self.transcribed_stmts.push(let_stmt);

        // change expr to the temporary variable path
        *expr = Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path: Path {
                leading_colon: None,
                segments: Punctuated::from_iter(vec![PathSegment {
                    ident: tmp_ident,
                    arguments: syn::PathArguments::None,
                }]),
            },
        });
        Ok(())
    }
}
