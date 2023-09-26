use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::{Comma, Paren};
use syn::visit_mut::VisitMut;
use syn::{Expr, ExprCall, ExprPath, Path};
use syn_path::path;

fn convert_to_call(expr: &mut Expr, path: Path, args: Punctuated<Expr, Comma>) {
    let func = Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path,
    });

    *expr = Expr::Call(ExprCall {
        attrs: vec![],
        func: Box::new(func),
        paren_token: Paren::default(),
        args,
    });
}

fn transcribe_expression(expr: &mut Expr) -> Result<(), anyhow::Error> {
    match expr {
        syn::Expr::Binary(binary) => {
            let path = match binary.op {
                syn::BinOp::Add(_) => path!(::std::ops::Add::add),
                syn::BinOp::Sub(_) => path!(::std::ops::Sub::sub),
                syn::BinOp::Mul(_) => path!(::std::ops::Mul::mul),
                syn::BinOp::Div(_) => path!(::std::ops::Div::div),
                syn::BinOp::Rem(_) => path!(::std::ops::Rem::rem),
                syn::BinOp::BitAnd(_) => path!(::std::ops::BitAnd::bitand),
                syn::BinOp::BitOr(_) => path!(::std::ops::BitOr::bitor),
                syn::BinOp::BitXor(_) => path!(::std::ops::BitXor::bitxor),
                syn::BinOp::Shl(_) => path!(::std::ops::Shl::shl),
                syn::BinOp::Shr(_) => path!(::std::ops::Shr::shr),
                _ => {
                    // conversion not supported, do nothing
                    return Ok(());
                }
            };
            let mut args: Punctuated<Expr, Comma> = Punctuated::new();
            args.push((*binary.left).clone());
            args.push((*binary.right).clone());
            convert_to_call(expr, path, args);
        }
        syn::Expr::Unary(unary) => {
            let path = match unary.op {
                syn::UnOp::Neg(_) => path!(::std::ops::Neg::neg),
                syn::UnOp::Not(_) => path!(::std::ops::Not::not),
                _ => {
                    // conversion not supported, do nothing
                    return Ok(());
                }
            };
            let mut args = Punctuated::new();
            args.push((*unary.expr).clone());
            convert_to_call(expr, path, args);
        }
        _ => {
            // do nothing
        }
    }
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
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Err(err) = transcribe_expression(expr) {
            if self.first_error.is_none() {
                self.first_error = Some(err.context(format!(
                    "Error converting '{}' to operation-free representation",
                    quote!(#expr)
                )));
            }
        }
        // delegate to transcribe nested expressions
        syn::visit_mut::visit_expr_mut(self, expr);
    }
}

pub fn transcribe(file: &mut syn::File) -> Result<(), anyhow::Error> {
    let mut visitor = Visitor::new();
    visitor.visit_file_mut(file);

    if let Some(first_error) = visitor.first_error {
        return Err(first_error);
    }
    Ok(())
}
