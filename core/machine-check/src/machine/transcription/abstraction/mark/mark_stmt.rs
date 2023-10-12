use anyhow::anyhow;
use proc_macro2::Span;
use quote::quote;
use syn::{
    punctuated::Punctuated, token::Comma, Expr, ExprCall, ExprField, ExprPath, ExprReference,
    Ident, Index, Member, Pat, PatWild, Path, PathArguments, PathSegment, Stmt,
};
use syn_path::path;

use crate::machine::transcription::util::{
    create_expr_path, create_expr_tuple, create_ident, create_let_stmt_from_ident_expr,
};

fn invert_fn_expr(fn_expr: &mut Expr) -> anyhow::Result<()> {
    let Expr::Path(fn_path) = fn_expr else {
        return Err(anyhow!("Inversion not implemented for called function expression {:?}", fn_expr));
    };
    if fn_path.qself.is_some() || fn_path.path.leading_colon.is_none() {
        return Err(anyhow!(
            "Inversion is not implemented for non-global or non-bare called function expressions"
        ));
    }

    let mut segments_iter = fn_path.path.segments.iter_mut();

    if let Some(crate_segment) = segments_iter.next() {
        let crate_ident = &mut crate_segment.ident;
        match crate_ident.to_string().as_str() {
            "std" => {}
            "mck" => {
                let Some(PathSegment {
                    ident: second_ident,
                    arguments: PathArguments::None,
                }) = segments_iter.next() else {
                    return Err(anyhow!("Inversion fail"));
                };
                *second_ident = Ident::new("refin", second_ident.span());
                return Ok(());
            }
            _ => (),
        }
    }
    Err(anyhow!(
        "Failed to invert function expression {}",
        quote!(#fn_expr)
    ))
}

pub fn create_join_stmt(left: Expr, right: Expr) -> Stmt {
    let left_mut_ref_expr = Expr::Reference(ExprReference {
        attrs: vec![],
        and_token: Default::default(),
        mutability: Some(Default::default()),
        expr: Box::new(left),
    });
    Stmt::Expr(
        Expr::Call(ExprCall {
            attrs: vec![],
            func: Box::new(Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: path!(::mck::refin::Join::apply_join),
            })),
            paren_token: Default::default(),
            args: Punctuated::from_iter(vec![left_mut_ref_expr, right]),
        }),
        Some(Default::default()),
    )
}

fn invert_call(stmts: &mut Vec<Stmt>, later_mark: ExprPath, call: &ExprCall) -> anyhow::Result<()> {
    // move function arguments to left
    let mut function_args = Punctuated::<Pat, Comma>::new();
    let mut all_args_wild = true;
    for arg in &call.args {
        let pat = match arg {
            Expr::Path(path) => {
                all_args_wild = false;
                Pat::Path(path.clone())
            }
            Expr::Lit(_) => Pat::Wild(PatWild {
                attrs: vec![],
                underscore_token: Default::default(),
            }),
            _ => {
                return Err(anyhow!(
                    "Inversion not implemented for function argument type {:?}",
                    arg
                ));
            }
        };
        function_args.push(pat);
    }

    if all_args_wild {
        // no effect
        return Ok(());
    }

    // change the function name
    let mut inverted_call = call.clone();
    invert_fn_expr(&mut inverted_call.func)?;
    // change the function parameters so that there is
    // the normal input tuple and normal output first
    // then mark later
    inverted_call.args.clear();
    let mut abstr_input_args = Punctuated::new();

    for arg in &call.args {
        match arg {
            Expr::Path(expr_path) => {
                let mut abstr_path = expr_path.clone();
                change_path_to_abstr(&mut abstr_path.path);
                abstr_input_args.push(Expr::Path(abstr_path))
            }
            Expr::Lit(_) => {
                // literal is passed unchanged
                abstr_input_args.push(arg.clone())
            }
            _ => {
                return Err(anyhow!(
                    "Inversion not implemented for function argument type {:?}",
                    arg
                ))
            }
        }
    }

    let abstr_input_arg = create_expr_tuple(abstr_input_args);
    inverted_call.args.push(abstr_input_arg);
    inverted_call.args.push(Expr::Path(later_mark));

    // construct the call statement and join each of earlier marks
    let tmp_name = format!("__mck_tmp_{}", stmts.len());
    let tmp_ident = create_ident(&tmp_name);

    stmts.push(create_let_stmt_from_ident_expr(
        tmp_ident.clone(),
        Expr::Call(inverted_call),
    ));

    for (index, arg) in call.args.iter().enumerate() {
        match arg {
            Expr::Path(expr_path) => {
                let left_path_expr = Expr::Path(expr_path.clone());
                let right_path_expr = Expr::Path(create_expr_path(Path::from(tmp_ident.clone())));
                let right_field_expr = Expr::Field(ExprField {
                    attrs: vec![],
                    base: Box::new(right_path_expr),
                    dot_token: Default::default(),
                    member: Member::Unnamed(Index {
                        index: index as u32,
                        span: Span::call_site(),
                    }),
                });
                // join instead of assigning to correctly remember values
                let stmt = create_join_stmt(left_path_expr, right_field_expr);

                stmts.push(stmt);
            }
            Expr::Lit(_) => {
                // do nothing
            }
            _ => {
                return Err(anyhow!(
                    "Inversion assignment not implemented for function argument type {:?}",
                    arg
                ))
            }
        }
    }

    Ok(())
}

fn change_path_to_abstr(path: &mut Path) {
    if path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments[0].arguments.is_none()
    {
        let ident = &mut path.segments[0].ident;
        if let Some(stripped_name) = ident.to_string().strip_prefix("__mck_mark_") {
            let abstr_name = format!("__mck_abstr_{}", stripped_name);
            *ident = Ident::new(&abstr_name, ident.span());
        }
    }
}

pub fn invert_simple_let(
    inverted_stmts: &mut Vec<Stmt>,
    left: &Pat,
    right: &Expr,
) -> anyhow::Result<()> {
    let later_mark = match left {
        Pat::Ident(left_pat_ident) => create_expr_path(Path::from(left_pat_ident.ident.clone())),
        Pat::Path(left_path) => left_path.clone(),
        _ => {
            return Err(anyhow!("Inversion not implemented for pattern {:?}", left));
        }
    };

    match right {
        Expr::Path(_) | Expr::Field(_) | Expr::Struct(_) => {
            let earlier_mark = right.clone();
            // join instead of assigning to preserve marking
            inverted_stmts.push(create_join_stmt(earlier_mark, Expr::Path(later_mark)));
            Ok(())
        }
        Expr::Call(expr_call) => invert_call(inverted_stmts, later_mark, expr_call),
        _ => Err(anyhow!(
            "Inversion not implemented for expression {:?}",
            right
        )),
    }
}

pub fn invert_stmt(inverted_stmts: &mut Vec<Stmt>, stmt: &Stmt) -> anyhow::Result<()> {
    let mut stmt = stmt.clone();
    match stmt {
        Stmt::Local(ref mut local) => {
            let Some(ref mut init) = local.init else {
                return Err(anyhow!("Inversion of non-initialized let is not supported"));
            };
            if init.diverge.is_some() {
                return Err(anyhow!("Inversion of diverging let not supported"));
            }
            let left = &local.pat;
            let right = init.expr.as_ref();
            invert_simple_let(inverted_stmts, left, right)
        }
        Stmt::Expr(Expr::Path(_), Some(_)) | Stmt::Expr(Expr::Struct(_), Some(_)) => {
            // no side effects, do not convert
            Ok(())
        }
        Stmt::Expr(_, _) | Stmt::Item(_) | Stmt::Macro(_) => Err(anyhow!(
            "Inversion of statement type {:?} not supported",
            stmt
        )),
    }
}
