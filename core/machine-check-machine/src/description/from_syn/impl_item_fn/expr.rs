use syn::{
    spanned::Spanned, Expr, ExprBinary, ExprCall, ExprField, ExprIndex, ExprReference, ExprStruct,
    ExprUnary, Member,
};
use syn_path::path;

use crate::{
    description::{from_syn::path::fold_path, DescriptionErrorType, Error},
    util::{create_expr_call, create_expr_path, ArgType},
    wir::{
        WArrayBaseExpr, WBasicType, WCallArg, WExpr, WExprCall, WExprField, WExprReference,
        WExprStruct, WIdent, WIndexedExpr, WIndexedIdent, WMacroableCallFunc, WStmt, WStmtAssign,
        ZTac,
    },
};

use super::FunctionFolder;

impl super::FunctionFolder {
    pub fn fold_right_expr(
        &mut self,
        expr: Expr,
        stmts: &mut Vec<WStmt<ZTac>>,
    ) -> Result<WIndexedExpr<WBasicType, WMacroableCallFunc<WBasicType>>, Error> {
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
    ) -> Result<WIdent, Error> {
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
    ) -> Result<WCallArg, Error> {
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
    ) -> Result<WIndexedExpr<WBasicType, WMacroableCallFunc<WBasicType>>, Error> {
        let expr_span = expr.span();
        Ok(match expr {
            Expr::Call(expr_call) => {
                WIndexedExpr::NonIndexed(WExpr::Call(self.fold_right_expr_call(expr_call)?))
            }
            Expr::Field(expr_field) => {
                WIndexedExpr::NonIndexed(WExpr::Field(self.fold_right_expr_field(expr_field)?))
            }
            Expr::Path(_) => {
                WIndexedExpr::NonIndexed(WExpr::Move(self.fn_folder.fold_expr_as_ident(expr)?))
            }
            Expr::Struct(expr_struct) => {
                WIndexedExpr::NonIndexed(WExpr::Struct(self.fold_right_expr_struct(expr_struct)?))
            }
            Expr::Reference(expr_reference) => WIndexedExpr::NonIndexed(WExpr::Reference(
                self.fold_right_expr_reference(expr_reference)?,
            )),
            Expr::Lit(expr_lit) => WIndexedExpr::NonIndexed(WExpr::Lit(expr_lit.lit)),
            Expr::Index(expr_index) => self.fold_right_expr_index(expr_index)?,
            Expr::Binary(expr_binary) => self.fold_right_expr(normalize_binary(expr_binary)?)?,
            Expr::Unary(expr_unary) => self.fold_right_expr(normalize_unary(expr_unary)?)?,
            _ => return Err(Error::unsupported_construct("Expression kind", expr_span)),
        })
    }

    fn fold_right_expr_call(
        &mut self,
        expr_call: ExprCall,
    ) -> Result<WExprCall<WMacroableCallFunc<WBasicType>>, Error> {
        {
            let fn_path = self.fn_folder.fold_expr_as_path(*expr_call.func)?;
            let mut args = Vec::new();
            for arg in expr_call.args {
                args.push(self.force_call_arg(arg)?);
            }

            Ok(WExprCall {
                fn_path: WMacroableCallFunc::Call(fn_path),
                args,
            })
        }
    }

    fn fold_right_expr_field(&mut self, expr_field: ExprField) -> Result<WExprField, Error> {
        let base = self.fn_folder.fold_expr_as_ident(*expr_field.base)?;
        let member = Self::extract_member(expr_field.member)?;
        Ok(WExprField { base, member })
    }

    fn fold_right_expr_struct(
        &mut self,
        expr_struct: ExprStruct,
    ) -> Result<WExprStruct<WBasicType>, Error> {
        if expr_struct.qself.is_some() {
            return Err(Error::unsupported_construct(
                "Quantified self",
                expr_struct.span(),
            ));
        }
        if expr_struct.rest.is_some() {
            return Err(Error::unsupported_construct(
                "Struct expressions with base",
                expr_struct.rest.span(),
            ));
        }

        let mut args = Vec::new();
        for field in expr_struct.fields {
            let member_ident = Self::extract_member(field.member)?;
            let member_value = self.force_ident(field.expr)?;
            args.push((member_ident, member_value))
        }

        Ok(WExprStruct {
            type_path: fold_path(expr_struct.path)?,
            fields: args,
        })
    }

    fn fold_right_expr_reference(
        &mut self,
        expr_reference: ExprReference,
    ) -> Result<WExprReference, Error> {
        Ok(match *expr_reference.expr {
            Expr::Path(expr_path) => {
                WExprReference::Ident(self.fn_folder.fold_expr_as_ident(Expr::Path(expr_path))?)
            }
            Expr::Field(expr_field) => {
                let member = Self::extract_member(expr_field.member)?;
                WExprReference::Field(WExprField {
                    base: self.force_ident(*expr_field.base)?,
                    member,
                })
            }
            _ => {
                return Err(Error::unsupported_construct(
                    "Expression kind inside reference",
                    expr_reference.expr.span(),
                ))
            }
        })
    }

    fn fold_right_expr_index(
        &mut self,
        expr_index: ExprIndex,
    ) -> Result<WIndexedExpr<WBasicType, WMacroableCallFunc<WBasicType>>, Error> {
        let array_base = match *expr_index.expr {
            Expr::Path(expr_path) => {
                WArrayBaseExpr::Ident(self.fn_folder.fold_expr_as_ident(Expr::Path(expr_path))?)
            }
            Expr::Field(expr_field) => {
                let field_base = self.force_ident(*expr_field.base)?;
                let member = Self::extract_member(expr_field.member)?;
                WArrayBaseExpr::Field(WExprField {
                    base: field_base,
                    member,
                })
            }
            _ => {
                return Err(Error::unsupported_construct(
                    "Expression kind as array base",
                    expr_index.expr.span(),
                ))
            }
        };
        let index_ident = self.force_ident(*expr_index.index)?;

        Ok(WIndexedExpr::Indexed(array_base, index_ident))
    }

    fn extract_member(member: Member) -> Result<WIdent, Error> {
        match member {
            Member::Named(ident) => Ok(WIdent::from_syn_ident(ident)),
            Member::Unnamed(index) => Err(Error::unsupported_construct(
                "Unnamed members",
                index.span(),
            )),
        }
    }

    fn force_call_arg(&mut self, expr: Expr) -> Result<WCallArg, Error> {
        if let Expr::Lit(lit) = expr {
            return Ok(WCallArg::Literal(lit.lit));
        }
        Ok(WCallArg::Ident(self.force_ident(expr)?))
    }

    fn force_ident(&mut self, expr: Expr) -> Result<WIdent, Error> {
        // try to fold the expression as ident first
        if let Ok(ident) = self.fn_folder.fold_expr_as_ident(expr.clone()) {
            return Ok(ident);
        }
        self.move_through_temp(expr)
    }

    fn move_through_temp(&mut self, expr: Expr) -> Result<WIdent, Error> {
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
                self.fold_right_expr(expr)?
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
        Ok(tmp_ident)
    }
}

fn normalize_unary(expr_unary: ExprUnary) -> Result<Expr, Error> {
    let span = expr_unary.op.span();
    let path = match expr_unary.op {
        syn::UnOp::Deref(_) => {
            return Err(Error::new(
                DescriptionErrorType::UnsupportedConstruct("Dereference"),
                span,
            ))
        }
        syn::UnOp::Not(_) => path!(::std::ops::Not::not),
        syn::UnOp::Neg(_) => path!(::std::ops::Neg::neg),
        _ => {
            return Err(Error::new(
                DescriptionErrorType::UnsupportedConstruct("Unary operator"),
                span,
            ))
        }
    };
    // construct the call
    Ok(create_expr_call(
        create_expr_path(path),
        vec![(ArgType::Normal, *expr_unary.expr)],
    ))
}

fn normalize_binary(expr_binary: ExprBinary) -> Result<Expr, Error> {
    let span = expr_binary.op.span();
    let call_func = match expr_binary.op {
        syn::BinOp::Add(_) => path!(::std::ops::Add::add),
        syn::BinOp::Sub(_) => path!(::std::ops::Sub::sub),
        syn::BinOp::Mul(_) => path!(::std::ops::Mul::mul),
        syn::BinOp::Div(_) => path!(::std::ops::Div::div),
        syn::BinOp::Rem(_) => path!(::std::ops::Rem::rem),
        syn::BinOp::And(_) => {
            return Err(Error::new(
                DescriptionErrorType::UnsupportedConstruct("Short-circuiting AND"),
                span,
            ))
        }
        syn::BinOp::Or(_) => {
            return Err(Error::new(
                DescriptionErrorType::UnsupportedConstruct("Short-circuiting OR"),
                span,
            ))
        }
        syn::BinOp::BitAnd(_) => path!(::std::ops::BitAnd::bitand),
        syn::BinOp::BitOr(_) => path!(::std::ops::BitOr::bitor),
        syn::BinOp::BitXor(_) => path!(::std::ops::BitXor::bitxor),
        syn::BinOp::Shl(_) => path!(::std::ops::Shl::shl),
        syn::BinOp::Shr(_) => path!(::std::ops::Shr::shr),
        syn::BinOp::Eq(_) => path!(::std::cmp::PartialEq::eq),
        syn::BinOp::Ne(_) => path!(::std::cmp::PartialEq::ne),
        syn::BinOp::Lt(_) => path!(::std::cmp::PartialOrd::lt),
        syn::BinOp::Le(_) => path!(::std::cmp::PartialOrd::le),
        syn::BinOp::Gt(_) => path!(::std::cmp::PartialOrd::gt),
        syn::BinOp::Ge(_) => path!(::std::cmp::PartialOrd::ge),
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
            return Err(Error::new(
                DescriptionErrorType::UnsupportedConstruct("Assignment operators"),
                span,
            ))
        }
        _ => {
            return Err(Error::new(
                DescriptionErrorType::UnsupportedConstruct("Binary operator"),
                span,
            ))
        }
    };
    Ok(create_expr_call(
        create_expr_path(call_func),
        vec![
            (ArgType::Normal, *expr_binary.left),
            (ArgType::Normal, *expr_binary.right),
        ],
    ))
}
