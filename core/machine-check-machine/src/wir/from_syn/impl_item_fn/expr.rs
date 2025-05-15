use syn::{
    spanned::Spanned, Expr, ExprCall, ExprField, ExprIndex, ExprReference, ExprStruct, Member,
};

use crate::wir::{
    from_syn::path::fold_global_path, WArrayBaseExpr, WBasicType, WCallArg, WExpr, WExprCall,
    WExprField, WExprReference, WExprStruct, WIdent, WIndexedExpr, WIndexedIdent,
    WMacroableCallFunc, WStmt, WStmtAssign, ZTac,
};

use super::FunctionFolder;

impl super::FunctionFolder {
    pub fn fold_right_expr(
        &mut self,
        expr: Expr,
        stmts: &mut Vec<WStmt<ZTac>>,
    ) -> WIndexedExpr<WBasicType, WMacroableCallFunc<WBasicType>> {
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

struct RightExprFolder<'a> {
    fn_folder: &'a mut FunctionFolder,
    stmts: &'a mut Vec<WStmt<ZTac>>,
}

impl RightExprFolder<'_> {
    pub fn fold_right_expr(
        &mut self,
        expr: Expr,
    ) -> WIndexedExpr<WBasicType, WMacroableCallFunc<WBasicType>> {
        match expr {
            Expr::Call(expr_call) => {
                WIndexedExpr::NonIndexed(WExpr::Call(self.fold_right_expr_call(expr_call)))
            }
            Expr::Field(expr_field) => {
                WIndexedExpr::NonIndexed(WExpr::Field(self.fold_right_expr_field(expr_field)))
            }
            Expr::Path(_) => {
                WIndexedExpr::NonIndexed(WExpr::Move(self.fn_folder.fold_expr_as_ident(expr)))
            }
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

    fn fold_right_expr_call(
        &mut self,
        expr_call: ExprCall,
    ) -> WExprCall<WMacroableCallFunc<WBasicType>> {
        {
            // the function path
            let fn_path = self.fn_folder.fold_expr_as_path(*expr_call.func);

            let mut args = Vec::new();
            for arg in expr_call.args {
                args.push(self.force_call_arg(arg));
            }

            WExprCall {
                fn_path: WMacroableCallFunc::Call(fn_path),
                args,
            }
        }
    }

    fn fold_right_expr_field(&mut self, expr_field: ExprField) -> WExprField {
        // the member is just a regular identifier
        let member = match expr_field.member {
            syn::Member::Named(ident) => WIdent::from_syn_ident(ident),
            syn::Member::Unnamed(_index) => panic!("Unnamed members not supported"),
        };
        WExprField {
            base: self.fn_folder.fold_expr_as_ident(*expr_field.base),
            member,
        }
    }

    fn fold_right_expr_struct(&mut self, expr_struct: ExprStruct) -> WExprStruct<WBasicType> {
        let args = expr_struct
            .fields
            .into_pairs()
            .map(|pair| {
                let field_value = pair.into_value();
                let left = match field_value.member {
                    syn::Member::Named(ident) => WIdent::from_syn_ident(ident),
                    syn::Member::Unnamed(_) => {
                        panic!("Unnamed struct members not supported")
                    }
                };
                let right = self.force_ident(field_value.expr);
                (left, right)
            })
            .collect();
        WExprStruct {
            type_path: fold_global_path(expr_struct.path),
            fields: args,
        }
    }

    fn fold_right_expr_reference(&mut self, expr_reference: ExprReference) -> WExprReference {
        match *expr_reference.expr {
            Expr::Path(expr_path) => {
                WExprReference::Ident(self.fn_folder.fold_expr_as_ident(Expr::Path(expr_path)))
            }
            Expr::Field(expr_field) => {
                let inner = match expr_field.member {
                    syn::Member::Named(ident) => WIdent::from_syn_ident(ident),
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

    fn fold_right_expr_index(
        &mut self,
        expr_index: ExprIndex,
    ) -> WIndexedExpr<WBasicType, WMacroableCallFunc<WBasicType>> {
        let array_base = match *expr_index.expr {
            Expr::Path(expr_path) => {
                WArrayBaseExpr::Ident(self.fn_folder.fold_expr_as_ident(Expr::Path(expr_path)))
            }
            Expr::Field(expr_field) => {
                let field_base = self.force_ident(*expr_field.base);

                let Member::Named(field_member) = expr_field.member else {
                    panic!("Unnamed members not supported");
                };
                WArrayBaseExpr::Field(WExprField {
                    base: field_base,
                    member: WIdent::from_syn_ident(field_member),
                })
            }
            _ => panic!("Expr index array should be ident or field"),
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
        match self.fn_folder.try_fold_expr_as_ident(expr) {
            Ok(ident) => ident,
            Err(expr) => self.move_through_temp(expr),
        }
    }

    fn move_through_temp(&mut self, expr: Expr) -> WIdent {
        let expr_span = expr.span();
        // process the expression first before moving it through temporary
        let expr = match expr {
            syn::Expr::Path(_) => {
                // just fold as ident
                return self.fn_folder.fold_expr_as_ident(expr);
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
