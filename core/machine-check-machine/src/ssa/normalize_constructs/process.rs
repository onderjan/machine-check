use syn::{
    punctuated::Punctuated, spanned::Spanned, Block, Expr, ExprAssign, ExprBinary, ExprBlock,
    ExprCall, ExprIf, ExprInfer, ExprUnary, Stmt,
};
use syn_path::path;

use crate::{
    support::local::extract_local_ident_with_type,
    util::{create_expr_call, create_expr_ident, create_expr_path, ArgType},
};

impl super::Visitor {
    pub(super) fn process_block(&mut self, block: &mut Block) {
        // process the statements, splitting locals with init to assign later
        let mut processed_stmts = Vec::new();
        let num_stmts = block.stmts.len();
        for (index, stmt) in block.stmts.drain(..).enumerate() {
            match stmt {
                Stmt::Local(mut local) => {
                    let (ident, _ty) = extract_local_ident_with_type(&local);
                    // split init to assignment
                    if let Some(init) = local.init.take() {
                        if let Some(diverge) = init.diverge {
                            self.push_error(
                                String::from("Diverging local not supported"),
                                diverge.1.span(),
                            );
                        }
                        let assign_stmt = Stmt::Expr(
                            Expr::Assign(ExprAssign {
                                attrs: vec![],
                                left: Box::new(create_expr_ident(ident)),
                                eq_token: init.eq_token,
                                right: init.expr,
                            }),
                            Some(local.semi_token),
                        );
                        processed_stmts.push(Stmt::Local(local));
                        processed_stmts.push(assign_stmt);
                    } else {
                        processed_stmts.push(Stmt::Local(local));
                    }
                }
                Stmt::Item(item) => {
                    // not supported
                    self.push_error(
                        String::from("Item inside statement not supported"),
                        item.span(),
                    );
                }
                Stmt::Expr(expr, mut semi) => {
                    // ensure it has semicolon if it is not the last statement
                    if semi.is_none() && index != num_stmts - 1 {
                        semi = Some(Default::default());
                    }
                    processed_stmts.push(Stmt::Expr(expr, semi));
                }
                Stmt::Macro(stmt_macro) => {
                    self.push_error(String::from("Used macro not supported"), stmt_macro.span());
                    processed_stmts.push(Stmt::Macro(stmt_macro));
                }
            }
        }
        block.stmts = processed_stmts;
    }

    pub(super) fn process_expr_if(&mut self, expr_if: &mut ExprIf) {
        // make sure it contains an else block
        if let Some((else_token, else_expr)) = expr_if.else_branch.take() {
            let else_expr = if matches!(*else_expr, Expr::Block(_)) {
                else_expr
            } else {
                // wrap the else expression inside a new block
                Box::new(Expr::Block(ExprBlock {
                    attrs: vec![],
                    label: None,
                    block: Block {
                        brace_token: Default::default(),
                        stmts: vec![Stmt::Expr(*else_expr, Some(Default::default()))],
                    },
                }))
            };
            expr_if.else_branch = Some((else_token, else_expr));
        } else {
            // create an empty else block
            expr_if.else_branch = Some((
                Default::default(),
                Box::new(Expr::Block(ExprBlock {
                    attrs: vec![],
                    label: None,
                    block: Block {
                        brace_token: Default::default(),
                        stmts: vec![],
                    },
                })),
            ));
        }

        // add call to Test if the condition is not a literal
        // do it after delegation so we do not trigger path error
        if !matches!(*expr_if.cond, Expr::Lit(_)) {
            let mut cond = Expr::Infer(ExprInfer {
                attrs: vec![],
                underscore_token: Default::default(),
            });
            std::mem::swap(&mut cond, &mut expr_if.cond);
            expr_if.cond = Box::new(Expr::Call(ExprCall {
                attrs: vec![],
                func: Box::new(create_expr_path(path!(::mck::concr::Test::into_bool))),
                paren_token: Default::default(),
                args: Punctuated::from_iter([cond]),
            }));
        }
    }

    pub(super) fn normalize_unary(&mut self, expr_unary: ExprUnary) -> Expr {
        let path = match expr_unary.op {
            syn::UnOp::Deref(_) => {
                self.push_error(
                    String::from("Dereference not supported"),
                    expr_unary.op.span(),
                );
                None
            }
            syn::UnOp::Not(_) => Some(path!(::std::ops::Not::not)),
            syn::UnOp::Neg(_) => Some(path!(::std::ops::Neg::neg)),
            _ => {
                self.push_error(String::from("Unknown unary operator"), expr_unary.op.span());
                None
            }
        };
        if let Some(path) = path {
            // construct the call
            create_expr_call(
                create_expr_path(path),
                vec![(ArgType::Normal, *expr_unary.expr)],
            )
        } else {
            // retain original if we were unsuccessful
            Expr::Unary(expr_unary)
        }
    }

    pub(super) fn normalize_binary(&mut self, expr_binary: ExprBinary) -> Expr {
        let call_func = match expr_binary.op {
            syn::BinOp::Add(_) => Some(path!(::std::ops::Add::add)),
            syn::BinOp::Sub(_) => Some(path!(::std::ops::Sub::sub)),
            syn::BinOp::Mul(_) => Some(path!(::std::ops::Mul::mul)),
            syn::BinOp::Div(_) => Some(path!(::std::ops::Div::div)),
            syn::BinOp::Rem(_) => Some(path!(::std::ops::Rem::rem)),
            syn::BinOp::And(_) => {
                self.push_error(
                    String::from("Short-circuiting AND not supported"),
                    expr_binary.op.span(),
                );
                None
            }
            syn::BinOp::Or(_) => {
                self.push_error(
                    String::from("Short-circuiting OR not supported"),
                    expr_binary.op.span(),
                );
                None
            }
            syn::BinOp::BitAnd(_) => Some(path!(::std::ops::BitAnd::bitand)),
            syn::BinOp::BitOr(_) => Some(path!(::std::ops::BitOr::bitor)),
            syn::BinOp::BitXor(_) => Some(path!(::std::ops::BitXor::bitxor)),
            syn::BinOp::Shl(_) => Some(path!(::std::ops::Shl::shl)),
            syn::BinOp::Shr(_) => Some(path!(::std::ops::Shr::shr)),
            syn::BinOp::Eq(_) => Some(path!(::std::cmp::PartialEq::eq)),
            syn::BinOp::Ne(_) => Some(path!(::std::cmp::PartialEq::ne)),
            syn::BinOp::Lt(_) => Some(path!(::std::cmp::PartialOrd::lt)),
            syn::BinOp::Le(_) => Some(path!(::std::cmp::PartialOrd::le)),
            syn::BinOp::Gt(_) => Some(path!(::std::cmp::PartialOrd::gt)),
            syn::BinOp::Ge(_) => Some(path!(::std::cmp::PartialOrd::ge)),
            syn::BinOp::AddAssign(_)
            | syn::BinOp::SubAssign(_)
            | syn::BinOp::MulAssign(_)
            | syn::BinOp::DivAssign(_)
            | syn::BinOp::RemAssign(_)
            | syn::BinOp::BitXorAssign(_)
            | syn::BinOp::BitAndAssign(_)
            | syn::BinOp::BitOrAssign(_)
            | syn::BinOp::ShlAssign(_)
            | syn::BinOp::ShrAssign(_) => {
                self.push_error(
                    String::from("Assignment operators not supported"),
                    expr_binary.op.span(),
                );
                None
            }
            _ => {
                self.push_error(
                    String::from("Unknown binary operator"),
                    expr_binary.op.span(),
                );
                None
            }
        };
        if let Some(path) = call_func {
            // construct the call
            create_expr_call(
                create_expr_path(path),
                vec![
                    (ArgType::Normal, *expr_binary.left),
                    (ArgType::Normal, *expr_binary.right),
                ],
            )
        } else {
            // retain original if we were unsuccessful
            Expr::Binary(expr_binary)
        }
    }
}
