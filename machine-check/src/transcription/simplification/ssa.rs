use anyhow::anyhow;
use proc_macro2::Span;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Let;
use syn::token::PathSep;
use syn::token::Semi;
use syn::Expr;
use syn::ExprPath;
use syn::Ident;
use syn::Local;
use syn::LocalInit;
use syn::Pat;
use syn::PatIdent;
use syn::Path;
use syn::PathSegment;
use syn::Token;
use syn::{visit_mut::VisitMut, Block, Stmt};

fn move_expression(expr: &mut Expr, transcribed_stmts: &mut Vec<Stmt>) {
    let tmp_ident = Ident::new(
        format!("__mck_tmp_{}", transcribed_stmts.len()).as_str(),
        Span::call_site(),
    );
    let mut tmp_ident_path_segments = Punctuated::<PathSegment, PathSep>::new();
    tmp_ident_path_segments.push(PathSegment {
        ident: tmp_ident.clone(),
        arguments: syn::PathArguments::None,
    });
    let tmp_ident_path = Path {
        leading_colon: None,
        segments: tmp_ident_path_segments,
    };
    let let_pattern = Pat::Ident(PatIdent {
        attrs: vec![],
        by_ref: None,
        mutability: None,
        ident: tmp_ident,
        subpat: None,
    });
    let let_init = LocalInit {
        eq_token: Token![=](expr.span()),
        expr: Box::new(expr.clone()),
        diverge: None,
    };
    // add new let statement to transcribed statements
    let let_stmt = Stmt::Local(Local {
        attrs: vec![],
        let_token: Let::default(),
        pat: let_pattern,
        init: Some(let_init),
        semi_token: Semi::default(),
    });
    transcribed_stmts.push(let_stmt);
    // change expr to the temporary identifier
    *expr = Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path: tmp_ident_path,
    });
}

fn force_move_expression(
    expr: &mut Expr,
    transcribed_stmts: &mut Vec<Stmt>,
) -> Result<(), anyhow::Error> {
    // transcribe expression first
    transcribe_expression(expr, transcribed_stmts)?;

    match expr {
        syn::Expr::Path(_) | syn::Expr::Lit(_) => {
            // do nothing, it is fine to leave as-is
        }
        _ => {
            // move
            move_expression(expr, transcribed_stmts);
        }
    }
    Ok(())
}

fn transcribe_expression(
    expr: &mut Expr,
    transcribed_stmts: &mut Vec<Stmt>,
) -> Result<(), anyhow::Error> {
    match expr {
        syn::Expr::Path(_) => {
            // do nothing
        }
        syn::Expr::Lit(_) => {
            // do nothing
        }
        syn::Expr::Field(field) => {
            // force-move base
            force_move_expression(&mut field.base, transcribed_stmts)?;
        }
        syn::Expr::Paren(paren) => {
            // force-move statement in parentheses
            force_move_expression(&mut paren.expr, transcribed_stmts)?;
            // remove parentheses
            *expr = (*paren.expr).clone();
        }
        syn::Expr::Call(call) => {
            // force-move call function expression and arguments
            force_move_expression(&mut call.func, transcribed_stmts)?;
            for arg in &mut call.args {
                force_move_expression(arg, transcribed_stmts)?;
            }
        }
        syn::Expr::Struct(expr_struct) => {
            // force-move statements and rest
            for field in &mut expr_struct.fields {
                force_move_expression(&mut field.expr, transcribed_stmts)?;
            }
            if let Some(ref mut rest) = expr_struct.rest {
                force_move_expression(rest, transcribed_stmts)?;
            }
        }
        _ => {
            return Err(anyhow!("Expression type {:?} not supported", expr));
        }
    }
    Ok(())
}

fn transcribe_statement(
    stmt: Stmt,
    transcribed_stmts: &mut Vec<Stmt>,
) -> Result<(), anyhow::Error> {
    let mut stmt = stmt;
    if let Stmt::Expr(ref mut expr, ref mut semi) = stmt {
        if semi.is_none() {
            // force movement from return expression for ease of use
            force_move_expression(expr, transcribed_stmts)?;
        } else {
            transcribe_expression(expr, transcribed_stmts)?;
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

    let expr = &mut *init.expr;
    transcribe_expression(expr, transcribed_stmts)?;
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

pub fn transcribe(file: &mut syn::File) -> Result<(), anyhow::Error> {
    // transcribe operations to calls first
    super::ops_to_calls::transcribe(file)?;

    let mut visitor = Visitor::new();

    visitor.visit_file_mut(file);

    if let Some(first_error) = visitor.first_error {
        return Err(first_error);
    }
    Ok(())
}
