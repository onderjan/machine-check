use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;
use syn::{Expr, ExprCall, ExprPath};
use syn_path::path;

pub fn apply(file: &mut syn::File) {
    struct Visitor();
    impl VisitMut for Visitor {
        fn visit_expr_mut(&mut self, expr: &mut Expr) {
            // apply transcription
            apply_to_expression(expr);
            // delegate to also apply transcription to nested expressions
            syn::visit_mut::visit_expr_mut(self, expr);
        }
    }
    Visitor().visit_file_mut(file);
}

fn apply_to_expression(expr: &mut Expr) {
    let (path, args) = match expr {
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
                    return;
                }
            };
            // convert binary operation expression to call
            let mut args = Punctuated::new();
            args.push((*binary.left).clone());
            args.push((*binary.right).clone());
            (path, args)
        }
        syn::Expr::Unary(unary) => {
            let path = match unary.op {
                syn::UnOp::Neg(_) => path!(::std::ops::Neg::neg),
                syn::UnOp::Not(_) => path!(::std::ops::Not::not),
                _ => {
                    // conversion not supported, do nothing
                    return;
                }
            };
            // convert unary operation expression to call
            let mut args = Punctuated::new();
            args.push((*unary.expr).clone());
            (path, args)
        }
        _ => {
            // do nothing
            return;
        }
    };
    // convert expression to call
    *expr = Expr::Call(ExprCall {
        attrs: vec![],
        func: Box::new(Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path,
        })),
        paren_token: Default::default(),
        args,
    });
}
