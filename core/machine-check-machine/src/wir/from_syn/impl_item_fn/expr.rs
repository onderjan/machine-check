use syn::{
    spanned::Spanned, Expr, ExprCall, ExprField, ExprIndex, ExprReference, ExprStruct, Member,
};

use crate::{
    util::{extract_expr_ident, extract_expr_path, extract_path_ident},
    wir::{
        WArrayBaseExpr, WBasicType, WCallArg, WExpr, WExprCall, WExprField, WExprReference,
        WExprStruct, WIdent, WIndexedExpr, WIndexedIdent, WStmt, WStmtAssign, ZTac,
    },
};

use super::FunctionFolder;

impl super::FunctionFolder<'_> {
    pub fn fold_right_expr(
        &mut self,
        expr: Expr,
        stmts: &mut Vec<WStmt<ZTac>>,
    ) -> WIndexedExpr<WBasicType> {
        RightExprFolder {
            fn_folder: self,
            stmts,
        }
        .fold_right_expr(expr)
    }

    pub fn force_right_expr_to_ident<'a>(
        &'a mut self,
        expr: Expr,
        stmts: &'a mut Vec<WStmt<ZTac>>,
    ) -> WIdent {
        {
            RightExprFolder {
                fn_folder: self,
                stmts,
            }
            .force_ident(expr)
        }
    }

    pub fn force_right_expr_to_call_arg<'a>(
        &'a mut self,
        expr: Expr,
        stmts: &'a mut Vec<WStmt<ZTac>>,
    ) -> WCallArg {
        {
            RightExprFolder {
                fn_folder: self,
                stmts,
            }
            .force_call_arg(expr)
        }
    }
}

struct RightExprFolder<'a, 'b> {
    fn_folder: &'a mut FunctionFolder<'b>,
    stmts: &'a mut Vec<WStmt<ZTac>>,
}

impl RightExprFolder<'_, '_> {
    pub fn fold_right_expr(&mut self, expr: Expr) -> WIndexedExpr<WBasicType> {
        match expr {
            Expr::Call(expr_call) => {
                WIndexedExpr::NonIndexed(WExpr::Call(self.fold_right_expr_call(expr_call)))
            }
            Expr::Field(expr_field) => {
                WIndexedExpr::NonIndexed(WExpr::Field(self.fold_right_expr_field(expr_field)))
            }
            Expr::Path(expr_path) => WIndexedExpr::NonIndexed(WExpr::Move(
                extract_path_ident(&expr_path.path)
                    .expect("Right expr path should be ident")
                    .clone()
                    .into(),
            )),
            Expr::Struct(expr_struct) => {
                WIndexedExpr::NonIndexed(WExpr::Struct(self.fold_right_expr_struct(expr_struct)))
            }
            Expr::Reference(expr_reference) => WIndexedExpr::NonIndexed(WExpr::Reference(
                self.fold_right_expr_reference(expr_reference),
            )),
            Expr::Lit(expr_lit) => WIndexedExpr::NonIndexed(WExpr::Lit(expr_lit.lit)),
            Expr::Index(expr_index) => self.fold_right_expr_index(expr_index),
            _ => panic!("Unexpected right expression {:?}", expr),
        }
    }

    fn fold_right_expr_call(&mut self, expr_call: ExprCall) -> WExprCall<WBasicType> {
        {
            let fn_path = extract_expr_path(&expr_call.func)
                .expect("Call function should be path")
                .clone()
                .into();

            let mut args = Vec::new();
            for arg in expr_call.args {
                args.push(self.force_call_arg(arg));
            }

            WExprCall { fn_path, args }
        }
    }

    fn fold_right_expr_field(&mut self, expr_field: ExprField) -> WExprField {
        let inner = match expr_field.member {
            syn::Member::Named(ident) => ident.into(),
            syn::Member::Unnamed(_index) => panic!("Unnamed members not supported"),
        };
        WExprField {
            base: extract_expr_ident(&expr_field.base)
                .expect("Field base should be ident")
                .clone()
                .into(),
            member: inner,
        }
    }

    fn fold_right_expr_struct(&mut self, expr_struct: ExprStruct) -> WExprStruct<WBasicType> {
        let args = expr_struct
            .fields
            .into_pairs()
            .map(|pair| {
                let field_value = pair.into_value();
                let left = match field_value.member {
                    syn::Member::Named(ident) => ident.into(),
                    syn::Member::Unnamed(_) => {
                        panic!("Unnamed struct members not supported")
                    }
                };
                let right = self.force_ident(field_value.expr);
                (left, right)
            })
            .collect();
        WExprStruct {
            type_path: expr_struct.path.into(),
            fields: args,
        }
    }

    fn fold_right_expr_reference(&mut self, expr_reference: ExprReference) -> WExprReference {
        match *expr_reference.expr {
            Expr::Path(expr_path) => {
                let Some(ident) = extract_path_ident(&expr_path.path) else {
                    panic!("Reference should be to an ident")
                };

                WExprReference::Ident(ident.clone().into())
            }
            Expr::Field(expr_field) => {
                let inner = match expr_field.member {
                    syn::Member::Named(ident) => ident.into(),
                    syn::Member::Unnamed(_index) => panic!("Unnamed members not supported"),
                };
                WExprReference::Field(WExprField {
                    base: self.force_ident(*expr_field.base),
                    member: inner,
                })
            }
            _ => panic!(
                "Unexpected expression inside reference {:?}",
                expr_reference.expr
            ),
        }
    }

    fn fold_right_expr_index(&mut self, expr_index: ExprIndex) -> WIndexedExpr<WBasicType> {
        let array_base = if let Some(array_ident) = extract_expr_ident(&expr_index.expr) {
            WArrayBaseExpr::Ident(array_ident.clone().into())
        } else {
            match *expr_index.expr {
                Expr::Field(expr_field) => {
                    let field_base = self.force_ident(*expr_field.base);

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
        let index_ident = self.force_ident(*expr_index.index);

        WIndexedExpr::Indexed(array_base, index_ident)
    }

    fn force_call_arg(&mut self, expr: Expr) -> WCallArg {
        if let Expr::Lit(lit) = expr {
            return WCallArg::Literal(lit.lit);
        }
        WCallArg::Ident(self.force_ident(expr))
    }

    fn force_ident(&mut self, expr: Expr) -> WIdent {
        if let Some(right) = extract_expr_ident(&expr) {
            return WIdent::from(right.clone());
        }
        self.move_through_temp(expr)
    }

    fn move_through_temp(&mut self, expr: Expr) -> WIdent {
        let expr_span = expr.span();
        // process the expression first before moving it through temporary
        let expr = match expr {
            syn::Expr::Path(_) => {
                // just extract the identifier
                let Some(ident) = extract_expr_ident(&expr) else {
                    panic!("Cannot extract identifier from path expression: {:?}", expr);
                };
                return WIdent::from(ident.clone());
            }
            syn::Expr::Paren(paren) => {
                // move statement in parentheses
                return self.move_through_temp(*paren.expr);
            }
            _ => {
                // fold the expression normally
                // so that nested expressions are properly converted to SSA
                self.fold_right_expr(expr)
            }
        };

        // create a temporary variable
        let tmp_ident = self
            .fn_folder
            .ident_creator
            .create_temporary_ident(expr_span);
        // add assignment statement; the temporary is only assigned to once here
        self.stmts.push(WStmt::Assign(WStmtAssign {
            left: WIndexedIdent::NonIndexed(tmp_ident.clone()),
            right: expr,
        }));

        // return the temporary variable ident
        tmp_ident
    }
}
