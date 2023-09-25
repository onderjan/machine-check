use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::Expr;
use syn::ExprInfer;
use syn::Pat;
use syn::Token;
use syn::{visit_mut::VisitMut, Block, Stmt};

fn transcribe_statement(
    mut stmt: Stmt,
    transcribed_stmts: &mut Vec<Stmt>,
) -> Result<(), anyhow::Error> {
    if let Stmt::Expr(Expr::Field(ref mut field), _) = stmt {
        // TODO: nested fields
        let Expr::Path(_) = *field.base else {
            return Err(anyhow!("Field statement not supported"));
        };

        transcribed_stmts.push(stmt);
        return Ok(());
    }
    if let Stmt::Expr(Expr::Path(_), _) = stmt {
        transcribed_stmts.push(stmt);
        return Ok(());
    }

    if let Stmt::Expr(Expr::Struct(ref mut expr_struct), _) = stmt {
        if expr_struct.dot2_token.is_some() || expr_struct.rest.is_some() {
            return Err(anyhow!("Struct expression statement rest not supported"));
        }
        for field in &expr_struct.fields {
            let Expr::Path(_) = field.expr else {
                return Err(anyhow!(
                    "Non-path member expression in struct expression statement field not supported"
                ));
            };
        }
        transcribed_stmts.push(stmt);
        return Ok(());
    }

    let Stmt::Local(ref mut local) = stmt else {
            return Err(anyhow!("Statement not supported"));
        };

    let Pat::Ident(ident) = &local.pat else {
            return Err(anyhow!(
                "Local let with non-ident pattern not supported"
            ));
        };
    if ident.by_ref.is_some() || ident.mutability.is_some() || ident.subpat.is_some() {
        return Err(anyhow!("Non-bare local let ident not supported"));
    }

    let Some(ref mut init) = local.init else {
            return Err(anyhow!(
                "Local let without init not supported"
            ));
        };

    if init.diverge.is_some() {
        return Err(anyhow!("Local let with diverging else not supported"));
    }

    let expr = &*init.expr;

    let expr = match expr {
        syn::Expr::Binary(_) => expr.clone(), /*Expr::Infer(ExprInfer {
        attrs: Vec::new(),
        underscore_token: Token![_](binary.span()),
        }),*/
        syn::Expr::Call(_) => expr.clone(),
        syn::Expr::Field(_) => expr.clone(),
        syn::Expr::Paren(_) => expr.clone(),
        syn::Expr::Path(_) => expr.clone(),
        syn::Expr::Unary(_) => expr.clone(),
        _ => {
            return Err(anyhow!("Expression type {:?} not supported", expr));
        }
    };
    *init.expr = expr;
    transcribed_stmts.push(stmt);

    Ok(())
}

fn transcribe_block(block: &mut Block) -> Result<(), anyhow::Error> {
    let mut transcribed_stmts = Vec::<Stmt>::new();

    for stmt in &block.stmts {
        let stmt_clone = stmt.clone();
        if let Err(err) = transcribe_statement(stmt_clone, &mut transcribed_stmts) {
            return Err(err.context(format!("Error transcribing '{}' to SSA", quote!(#stmt))));
        }
    }
    block.stmts = transcribed_stmts;

    Ok(())
}

struct Visitor {
    first_error: Option<anyhow::Error>,
}

impl Visitor {
    fn new() -> Visitor {
        Visitor { first_error: None }
    }
}

impl VisitMut for Visitor {
    fn visit_block_mut(&mut self, block: &mut Block) {
        if let Err(err) = transcribe_block(block) {
            if self.first_error.is_none() {
                self.first_error = Some(err);
            }
        }
        // do not delegate, the whole block is transcribed
    }
}

pub fn transcribe(concrete_machine: TokenStream) -> Result<TokenStream, anyhow::Error> {
    let mut file: syn::File = syn::parse2(concrete_machine)?;

    let mut visitor = Visitor::new();

    visitor.visit_file_mut(&mut file);

    if let Some(first_error) = visitor.first_error {
        return Err(first_error);
    }
    Ok(file.to_token_stream())
}
