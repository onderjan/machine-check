use syn::{Expr, Member};

use crate::{
    util::{extract_expr_ident, extract_expr_path, extract_path_ident},
    wir::{
        WArrayBaseExpr, WBasicType, WCallArg, WExpr, WExprCall, WExprField, WExprReference,
        WExprStruct, WIndexedExpr,
    },
};

pub fn fold_right_expr(expr: Expr) -> WIndexedExpr<WBasicType> {
    match expr {
        Expr::Call(expr_call) => {
            let args = expr_call
                .args
                .iter()
                .map(|arg| match arg {
                    Expr::Lit(expr) => WCallArg::Literal(expr.lit.clone()),
                    Expr::Path(expr) => {
                        WCallArg::Ident(extract_path_ident(&expr.path).unwrap().clone().into())
                    }
                    _ => panic!(
                        "Unexpected non-literal and non-path call argument: {:?}",
                        arg
                    ),
                })
                .collect();

            WIndexedExpr::NonIndexed(WExpr::Call(WExprCall {
                fn_path: extract_expr_path(&expr_call.func).unwrap().clone().into(),
                args,
            }))
        }
        Expr::Field(expr_field) => {
            let inner = match expr_field.member {
                syn::Member::Named(ident) => ident.into(),
                syn::Member::Unnamed(_index) => panic!("Unnamed members not supported"),
            };
            WIndexedExpr::NonIndexed(WExpr::Field(WExprField {
                base: extract_expr_ident(&expr_field.base).unwrap().clone().into(),
                member: inner,
            }))
        }
        Expr::Path(expr_path) => WIndexedExpr::NonIndexed(WExpr::Move(
            extract_path_ident(&expr_path.path).unwrap().clone().into(),
        )),
        Expr::Struct(expr_struct) => {
            let args = expr_struct
                .fields
                .into_pairs()
                .map(|pair| {
                    let field_value = pair.into_value();
                    let left = match field_value.member {
                        syn::Member::Named(ident) => ident.into(),
                        syn::Member::Unnamed(_) => panic!("Unnamed struct members not supported"),
                    };
                    let right = extract_expr_ident(&field_value.expr)
                        .unwrap()
                        .clone()
                        .into();
                    (left, right)
                })
                .collect();
            WIndexedExpr::NonIndexed(WExpr::Struct(WExprStruct {
                type_path: expr_struct.path.into(),
                fields: args,
            }))
        }
        Expr::Reference(expr_reference) => match *expr_reference.expr {
            Expr::Path(expr_path) => {
                let Some(ident) = extract_path_ident(&expr_path.path) else {
                    panic!("Reference should be to an ident")
                };

                WIndexedExpr::NonIndexed(WExpr::Reference(WExprReference::Ident(
                    ident.clone().into(),
                )))
            }
            Expr::Field(expr_field) => {
                let inner = match expr_field.member {
                    syn::Member::Named(ident) => ident.into(),
                    syn::Member::Unnamed(_index) => panic!("Unnamed members not supported"),
                };
                WIndexedExpr::NonIndexed(WExpr::Reference(WExprReference::Field(WExprField {
                    base: extract_expr_ident(&expr_field.base).unwrap().clone().into(),
                    member: inner,
                })))
            }
            _ => panic!(
                "Unexpected expression inside reference {:?}",
                expr_reference.expr
            ),
        },
        Expr::Lit(expr_lit) => WIndexedExpr::NonIndexed(WExpr::Lit(expr_lit.lit)),
        Expr::Index(expr_index) => {
            let array_base = if let Some(array_ident) = extract_expr_ident(&expr_index.expr) {
                WArrayBaseExpr::Ident(array_ident.clone().into())
            } else {
                match *expr_index.expr {
                    Expr::Field(expr_field) => {
                        let field_base = extract_expr_ident(&expr_field.base)
                            .expect("Indexed field base should be ident")
                            .clone()
                            .into();

                        let Member::Named(field_member) = expr_field.member else {
                            panic!("Unnamed members not supported");
                        };
                        WArrayBaseExpr::Field(WExprField {
                            base: field_base,
                            member: field_member.into(),
                        })
                    }
                    _ => panic!("Expr index array should be ident or field"),
                }
            };
            let index_ident = extract_expr_ident(&expr_index.index)
                .expect("Expr index should be ident")
                .clone()
                .into();

            WIndexedExpr::Indexed(array_base, index_ident)
        }
        _ => panic!("Unexpected right expression {:?}", expr),
    }
}
