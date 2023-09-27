use anyhow::anyhow;
use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, Expr, ExprAssign, ExprPath, ExprTuple, FieldPat, Ident, Local,
    LocalInit, Pat, PatStruct, PatTuple, PatWild, Path, PathArguments, PathSegment, Stmt,
};

use super::mark_ident::IdentVisitor;

pub fn convert_to_let_binding(bind_ident: Ident, stmt: &mut Stmt) -> anyhow::Result<()> {
    let Stmt::Expr(Expr::Path(expr_path), None) = stmt else {
        return Err(anyhow!("Functions without end result expression not supported"));
    };

    let local_init = LocalInit {
        eq_token: Default::default(),
        expr: Box::new(Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path: Path::from(bind_ident),
        })),
        diverge: None,
    };

    *stmt = Stmt::Local(Local {
        attrs: vec![],
        let_token: Default::default(),
        pat: Pat::Path(expr_path.clone()),
        init: Some(local_init),
        semi_token: Default::default(),
    });
    Ok(())
}

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
    let add_mark_segment = match segments_iter.next() {
        Some(PathSegment {
            ident: ref mut crate_ident,
            arguments: PathArguments::None,
        }) => {
            let crate_ident_string = crate_ident.to_string();
            match crate_ident_string.as_str() {
                "std" => {
                    let Some(PathSegment {
                        ident: second_ident,
                        arguments: PathArguments::None,
                    }) = segments_iter.next() else {
                        return Err(anyhow!("Inversion fail"));
                    };
                    *crate_ident = Ident::new("mck", crate_ident.span());
                    *second_ident = Ident::new("mark", second_ident.span());
                    false
                }
                "mck" => true,
                _ => return Err(anyhow!("Inversion fail")),
            }
        }
        _ => {
            return Err(anyhow!("Inversion fail"));
        }
    };

    if add_mark_segment {
        fn_path.path.segments.insert(
            1,
            PathSegment {
                ident: Ident::new("mark", Span::call_site()),
                arguments: PathArguments::None,
            },
        );
    }

    Ok(())
}

enum PatOrExpr {
    Pat(Pat),
    Expr(Expr),
}

fn invert(
    left: &Pat,
    right: &Expr,
    ident_visitor: &IdentVisitor,
    mark_ident_visitor: &IdentVisitor,
) -> anyhow::Result<Option<(PatOrExpr, Expr)>> {
    let mut new_right_expr = match left {
        Pat::Ident(left_pat_ident) => {
            let left_path = ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: Punctuated::from_iter(vec![PathSegment {
                        ident: left_pat_ident.ident.clone(),
                        arguments: syn::PathArguments::None,
                    }]),
                },
            };
            Expr::Path(left_path)
        }
        Pat::Path(left_path) => Expr::Path(left_path.clone()),
        _ => {
            return Err(anyhow!("Inversion not implemented for pattern {:?}", left));
        }
    };

    let new_left_pat = match right {
        Expr::Path(right_path) => Pat::Path(right_path.clone()),
        Expr::Call(right_call) => {
            // move function arguments to left
            let mut function_args = Vec::<Pat>::new();
            let mut all_args_wild = true;
            for arg in &right_call.args {
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
                            "Inversion not implemented for non-path function argument {:?}",
                            arg
                        ));
                    }
                };
                function_args.push(pat);
            }
            if all_args_wild {
                return Ok(None);
            }

            let mut tuple_elems = Punctuated::from_iter(function_args);
            if !tuple_elems.empty_or_trailing() {
                tuple_elems.push_punct(Default::default());
            }

            let new_left_pat = Pat::Tuple(PatTuple {
                attrs: vec![],
                paren_token: Default::default(),
                elems: tuple_elems,
            });

            // create reversal function in new right expression
            let mut new_right_call_expr = right_call.clone();
            // change the function name
            invert_fn_expr(&mut new_right_call_expr.func)?;
            // change the function parameters so that there is
            // the normal input tuple and normal output first
            // then mark later
            new_right_call_expr.args.clear();
            let mark_input_arg = new_right_expr.clone();
            let normal_input_args = right_call
                .args
                .iter()
                .map(|arg| {
                    if let Expr::Path(expr_path) = arg {
                        let mut path = expr_path.clone();
                        ident_visitor.apply_transcription_to_path(&mut path.path);
                        Expr::Path(path)
                    } else {
                        arg.clone()
                    }
                })
                .collect();

            let normal_input_arg = Expr::Tuple(ExprTuple {
                attrs: vec![],
                paren_token: Default::default(),
                elems: normal_input_args,
            });
            new_right_call_expr.args.push(normal_input_arg);

            let mut normal_output_arg = new_right_expr;
            if let Expr::Path(expr_path) = &mut normal_output_arg {
                ident_visitor.apply_transcription_to_path(&mut expr_path.path);
            }
            new_right_call_expr.args.push(normal_output_arg);

            new_right_call_expr.args.push(mark_input_arg);

            new_right_expr = Expr::Call(new_right_call_expr);
            new_left_pat
        }
        Expr::Struct(expr_struct) => {
            if expr_struct.rest.is_some() {
                return Err(anyhow!("Rest not supported"));
            }

            let mut field_pats = Vec::<FieldPat>::new();
            for field in &expr_struct.fields {
                let Expr::Path(expr_path) = &field.expr else {
                    return Err(anyhow!("Non-path field values not supported"));
                };

                let field_pat = FieldPat {
                    attrs: field.attrs.clone(),
                    member: field.member.clone(),
                    colon_token: field.colon_token,
                    pat: Box::new(Pat::Path(expr_path.clone())),
                };
                field_pats.push(field_pat);
            }

            Pat::Struct(PatStruct {
                attrs: expr_struct.attrs.clone(),
                qself: expr_struct.qself.clone(),
                path: expr_struct.path.clone(),
                brace_token: expr_struct.brace_token,
                fields: Punctuated::from_iter(field_pats),
                rest: None,
            })
        }
        Expr::Field(field) => {
            let mut field = field.clone();
            let Expr::Path(ref mut expr_path) = *field.base else {
                return Err(anyhow!("Non-path field base not supported"));
            };
            mark_ident_visitor.apply_transcription_to_path(&mut expr_path.path);

            let new_left_expr = Expr::Field(field);
            return Ok(Some((PatOrExpr::Expr(new_left_expr), new_right_expr)));
        }
        _ => {
            return Err(anyhow!(
                "Inversion not implemented for expression {:?}",
                right
            ));
        }
    };
    Ok(Some((PatOrExpr::Pat(new_left_pat), new_right_expr)))
}

pub fn invert_stmt(
    stmt: &Stmt,
    ident_visitor: &IdentVisitor,
    mark_ident_visitor: &IdentVisitor,
) -> anyhow::Result<Option<Stmt>> {
    let mut stmt = stmt.clone();
    Ok(match stmt {
        Stmt::Local(ref mut local) => {
            let Some(ref mut init) = local.init else {
                return Ok(Some(stmt));
            };
            if init.diverge.is_some() {
                return Err(anyhow!(
                    "Inversion of diverging let-statement not supported"
                ));
            }
            let original_left = &local.pat;
            let original_right = init.expr.as_ref();
            let inverted = invert(
                original_left,
                original_right,
                ident_visitor,
                mark_ident_visitor,
            )?;
            let Some((left, right)) = inverted else {
                return Ok(None);
            };
            Some(match left {
                PatOrExpr::Pat(left) => {
                    local.pat = left;
                    *init.expr = right;
                    stmt
                }
                PatOrExpr::Expr(left) => Stmt::Expr(
                    Expr::Assign(ExprAssign {
                        attrs: local.attrs.clone(),
                        left: Box::new(left),
                        eq_token: Default::default(),
                        right: Box::new(right),
                    }),
                    Some(Default::default()),
                ),
            })
        }
        Stmt::Expr(Expr::Path(_), _) => {
            // no side effects, just lose
            None
        }
        Stmt::Expr(_, _) | Stmt::Item(_) | Stmt::Macro(_) => {
            return Err(anyhow!(
                "Inversion of statement type {:?} not supported",
                stmt
            ));
        }
    })
}
