use std::str::FromStr;

use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, Expr, ExprBinary, ExprCall, ExprField,
    ExprIndex, ExprLit, ExprReference, ExprStruct, ExprUnary, GenericArgument, Lit, Member, Path,
    PathArguments, PathSegment,
};
use syn_path::path;

use crate::{
    description::{
        from_syn::{path::fold_path, ty::fold_type},
        Error, ErrorType,
    },
    util::{create_expr_call, create_expr_path, ArgType},
    wir::{
        WArrayBaseExpr, WBasicType, WCall, WCallArg, WExpr, WExprField, WExprHighCall,
        WExprReference, WExprStruct, WHighMckExt, WHighMckNew, WHighStdInto, WHighStdIntoType,
        WIdent, WIndexedExpr, WIndexedIdent, WMacroableStmt, WReference, WStdBinary, WStdBinaryOp,
        WStdUnary, WStdUnaryOp, WStmtAssign, WType, WTypeArray, ZTac, MCK_HIGH_BITVECTOR_ARRAY_NEW,
        MCK_HIGH_BITVECTOR_NEW, MCK_HIGH_EXT, MCK_HIGH_SIGNED_NEW, MCK_HIGH_UNSIGNED_NEW,
        STD_CLONE, STD_INTO,
    },
};

use super::FunctionFolder;

impl super::FunctionFolder {
    pub fn fold_right_expr(
        &mut self,
        expr: Expr,
        stmts: &mut Vec<WMacroableStmt<ZTac>>,
    ) -> Result<WIndexedExpr<WExprHighCall>, Error> {
        RightExprFolder {
            fn_folder: self,
            stmts,
        }
        .fold_right_expr(expr)
    }

    pub fn force_right_expr_to_ident<'a>(
        &'a mut self,
        expr: Expr,
        stmts: &'a mut Vec<WMacroableStmt<ZTac>>,
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
        stmts: &'a mut Vec<WMacroableStmt<ZTac>>,
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
    stmts: &'a mut Vec<WMacroableStmt<ZTac>>,
}

impl RightExprFolder<'_> {
    pub fn fold_right_expr(&mut self, expr: Expr) -> Result<WIndexedExpr<WExprHighCall>, Error> {
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
            Expr::Paren(expr_paren) => {
                // just fold the inside
                self.fold_right_expr(*expr_paren.expr)?
            }
            Expr::Group(expr_group) => {
                // just fold the inside
                self.fold_right_expr(*expr_group.expr)?
            }
            _ => return Err(Error::unsupported_construct("Expression kind", expr_span)),
        })
    }

    fn fold_right_expr_call(&mut self, expr_call: ExprCall) -> Result<WExprHighCall, Error> {
        let Expr::Path(expr_path) = &*expr_call.func else {
            return Err(Error::unsupported_construct(
                "Non-path function operand",
                expr_call.span(),
            ));
        };
        if expr_path.qself.is_some() {
            return Err(Error::unsupported_construct(
                "Qualified self in function operand",
                expr_path.span(),
            ));
        }

        let fn_path = &expr_path.path;
        let fn_path_span = fn_path.span();

        let mut nongeneric_path_string = if fn_path.leading_colon.is_some() {
            String::from("::")
        } else {
            String::new()
        };

        let mut first = true;

        for pair in fn_path.segments.pairs() {
            if first {
                first = false;
            } else {
                nongeneric_path_string += "::";
            }
            let segment = pair.into_value();
            nongeneric_path_string += &segment.ident.to_string();
        }

        // TODO: generics

        if let Ok(unary_op) = WStdUnaryOp::from_str(&nongeneric_path_string) {
            return self.create_std_unary(unary_op, fn_path, expr_call.args);
        }
        if let Ok(binary_op) = WStdBinaryOp::from_str(&nongeneric_path_string) {
            return self.create_std_binary(binary_op, fn_path, expr_call.args);
        }
        match nongeneric_path_string.as_str() {
            MCK_HIGH_EXT => {
                return self.create_mck_ext(fn_path, expr_call.args);
            }
            MCK_HIGH_BITVECTOR_NEW
            | MCK_HIGH_UNSIGNED_NEW
            | MCK_HIGH_SIGNED_NEW
            | MCK_HIGH_BITVECTOR_ARRAY_NEW => {
                return self.create_mck_new(fn_path, expr_call.args);
            }
            STD_CLONE => {
                return self.create_std_clone(fn_path, expr_call.args);
            }
            STD_INTO => {
                return self.create_std_into(fn_path, expr_call.args);
            }
            _ => {}
        }

        let fn_path = fold_path(fn_path.clone(), Some(&self.fn_folder.self_ty))?;
        // ensure it is not a local-scope ident
        if !fn_path.leading_colon && fn_path.segments.len() == 1 {
            let ident = &fn_path.segments[0].ident;
            if self.fn_folder.lookup_local_ident(ident).is_some() {
                return Err(Error::unsupported_construct(
                    "Local ident as function operand",
                    fn_path_span.span(),
                ));
            }
        }
        let mut args = Vec::new();
        for arg in expr_call.args {
            args.push(self.force_call_arg(arg)?);
        }

        Ok(WExprHighCall::Call(WCall { fn_path, args }))
    }

    fn create_std_unary(
        &mut self,
        op: WStdUnaryOp,
        fn_path: &Path,
        args: Punctuated<Expr, Comma>,
    ) -> Result<WExprHighCall, Error> {
        Self::assure_nongeneric_fn_path(fn_path)?;
        let operand = self.parse_single_ident_arg(args)?;
        Ok(WExprHighCall::StdUnary(WStdUnary { op, operand }))
    }

    fn create_std_binary(
        &mut self,
        op: WStdBinaryOp,
        fn_path: &Path,
        args: Punctuated<Expr, Comma>,
    ) -> Result<WExprHighCall, Error> {
        Self::assure_nongeneric_fn_path(fn_path)?;
        let (a, b) = self.parse_two_ident_args(args)?;
        Ok(WExprHighCall::StdBinary(WStdBinary { op, a, b }))
    }

    fn create_mck_ext(
        &mut self,
        fn_path: &Path,
        args: Punctuated<Expr, Comma>,
    ) -> Result<WExprHighCall, Error> {
        let mut fn_path = fn_path.clone();

        let second_segment = &mut fn_path.segments[1];
        let width = Self::parse_single_u32_generics(second_segment)?;
        second_segment.arguments = syn::PathArguments::None;

        Self::assure_nongeneric_fn_path(&fn_path)?;
        let from = self.parse_single_ident_arg(args)?;
        Ok(WExprHighCall::MckExt(WHighMckExt { width, from }))
    }

    fn create_mck_new(
        &mut self,
        fn_path: &Path,
        args: Punctuated<Expr, Comma>,
    ) -> Result<WExprHighCall, Error> {
        let mut fn_path = fn_path.clone();
        let second_segment = &mut fn_path.segments[1];

        if second_segment.ident.to_string().as_str() == "BitvectorArray" {
            // TODO: construct bitvector array as a test
            let (index_width, element_width) = Self::parse_two_u32_generics(second_segment)?;
            let fill_ident = self.parse_single_ident_arg(args)?;

            return Ok(WExprHighCall::MckNew(WHighMckNew::BitvectorArray(
                WTypeArray {
                    index_width,
                    element_width,
                },
                fill_ident,
            )));
        }

        let width = Self::parse_single_u32_generics(second_segment)?;
        second_segment.arguments = syn::PathArguments::None;

        let value = self.parse_single_const_arg(args)?;

        let kind = match second_segment.ident.to_string().as_str() {
            "Bitvector" => WHighMckNew::Bitvector(width, value),
            "Unsigned" => WHighMckNew::Unsigned(width, value),
            "Signed" => WHighMckNew::Signed(width, value),
            _ => panic!("Unexpected function path here"),
        };

        Self::assure_nongeneric_fn_path(&fn_path)?;

        Ok(WExprHighCall::MckNew(kind))
    }

    fn create_std_into(
        &mut self,
        fn_path: &Path,
        args: Punctuated<Expr, Comma>,
    ) -> Result<WExprHighCall, Error> {
        let mut fn_path = fn_path.clone();
        let third_segment = &mut fn_path.segments[2];

        let ty = self.parse_single_type_generics(third_segment)?;
        third_segment.arguments = syn::PathArguments::None;

        let WReference::None = ty.reference else {
            return Err(Error::unsupported_construct(
                "Reference type",
                third_segment.span(),
            ));
        };

        let ty = match ty.inner {
            WBasicType::Bitvector(width) => WHighStdIntoType::Bitvector(width),
            WBasicType::Unsigned(width) => WHighStdIntoType::Unsigned(width),
            WBasicType::Signed(width) => WHighStdIntoType::Signed(width),
            _ => {
                return Err(Error::unsupported_construct(
                    "Non-bitvector type",
                    third_segment.span(),
                ))
            }
        };

        let from = self.parse_single_ident_arg(args)?;
        Ok(WExprHighCall::StdInto(WHighStdInto { ty, from }))
    }

    fn parse_single_u32_generics(segment: &PathSegment) -> Result<u32, Error> {
        let turbofished = Self::extract_turbofished(segment)?;
        if turbofished.len() != 1 {
            return Err(Error::new(
                ErrorType::IllegalConstruct(String::from(
                    "Exactly one generic argument should be used here",
                )),
                segment.span(),
            ));
        }

        Self::parse_u32_generic(&turbofished[0])
    }

    fn parse_two_u32_generics(segment: &PathSegment) -> Result<(u32, u32), Error> {
        let turbofished = Self::extract_turbofished(segment)?;
        if turbofished.len() != 2 {
            return Err(Error::new(
                ErrorType::IllegalConstruct(String::from(
                    "Exactly 2 generic arguments should be used here",
                )),
                segment.span(),
            ));
        }

        let first = Self::parse_u32_generic(&turbofished[0])?;
        let second = Self::parse_u32_generic(&turbofished[1])?;
        Ok((first, second))
    }

    fn parse_single_type_generics(
        &self,
        segment: &PathSegment,
    ) -> Result<WType<WBasicType>, Error> {
        let turbofished = Self::extract_turbofished(segment)?;
        if turbofished.len() != 1 {
            return Err(Error::new(
                ErrorType::IllegalConstruct(String::from(
                    "Exactly one generic argument should be used here",
                )),
                segment.span(),
            ));
        }

        let arg = &turbofished[0];
        let GenericArgument::Type(arg) = arg else {
            return Err(Error::unsupported_construct(
                "Non-type generic argument",
                segment.span(),
            ));
        };

        let ty = fold_type(arg.clone(), Some(&self.fn_folder.self_ty))?;
        Ok(ty)
    }

    fn extract_turbofished(
        segment: &PathSegment,
    ) -> Result<&Punctuated<GenericArgument, Comma>, Error> {
        let PathArguments::AngleBracketed(generic_args) = &segment.arguments else {
            return Err(Error::unsupported_construct(
                "This call without generic argument",
                segment.span(),
            ));
        };
        if generic_args.colon2_token.is_none() {
            return Err(Error::new(
                ErrorType::IllegalConstruct(String::from("Turbofish should be used here")),
                segment.span(),
            ));
        }
        Ok(&generic_args.args)
    }

    fn parse_u32_generic(arg: &GenericArgument) -> Result<u32, Error> {
        let GenericArgument::Const(Expr::Lit(ExprLit {
            lit: Lit::Int(lit_int),
            ..
        })) = arg
        else {
            return Err(Error::new(
                ErrorType::IllegalConstruct(String::from(
                    "The generic argument here should be a literal",
                )),
                arg.span(),
            ));
        };

        let result = lit_int.base10_parse();
        let Ok(result) = result else {
            return Err(Error::new(
                ErrorType::IllegalConstruct(String::from(
                    "The generic argument here should be parseable as u32",
                )),
                arg.span(),
            ));
        };
        Ok(result)
    }

    fn create_std_clone(
        &mut self,
        fn_path: &Path,
        args: Punctuated<Expr, Comma>,
    ) -> Result<WExprHighCall, Error> {
        Self::assure_nongeneric_fn_path(fn_path)?;
        let ident = self.parse_single_ident_arg(args)?;
        Ok(WExprHighCall::StdClone(ident))
    }

    fn parse_single_const_arg(&mut self, args: Punctuated<Expr, Comma>) -> Result<i128, Error> {
        let span = args.span();
        if args.len() != 1 {
            return Err(Error::new(
                ErrorType::IllegalConstruct(String::from("Exactly 1 argument expected")),
                span,
            ));
        };
        let Expr::Lit(ExprLit {
            lit: Lit::Int(lit_int),
            attrs: _attrs,
        }) = args.into_iter().next().unwrap()
        else {
            return Err(Error::unsupported_construct(
                "Non-integer-literal argument here",
                span,
            ));
        };
        lit_int.base10_parse().map_err(|_| {
            Error::new(
                ErrorType::IllegalConstruct(String::from("Argument not parseable as i128")),
                span,
            )
        })
    }

    fn parse_single_ident_arg(&mut self, args: Punctuated<Expr, Comma>) -> Result<WIdent, Error> {
        if args.len() != 1 {
            return Err(Error::new(
                ErrorType::IllegalConstruct(String::from("Exactly 1 argument expected")),
                args.span(),
            ));
        };
        self.force_ident(args.into_iter().next().unwrap())
    }

    fn parse_two_ident_args(
        &mut self,
        args: Punctuated<Expr, Comma>,
    ) -> Result<(WIdent, WIdent), Error> {
        if args.len() != 2 {
            return Err(Error::new(
                ErrorType::IllegalConstruct(String::from("Exactly 2 arguments expected")),
                args.span(),
            ));
        };
        let mut iter = args.into_iter();
        let a = self.force_ident(iter.next().unwrap())?;
        let b = self.force_ident(iter.next().unwrap())?;
        Ok((a, b))
    }

    fn assure_nongeneric_fn_path(fn_path: &Path) -> Result<(), Error> {
        for segment in &fn_path.segments {
            if !segment.arguments.is_none() {
                return Err(Error::unsupported_construct(
                    "Unexpected generics",
                    segment.span(),
                ));
            };
        }
        Ok(())
    }

    fn fold_right_expr_field(&mut self, expr_field: ExprField) -> Result<WExprField, Error> {
        let base = self.fn_folder.fold_expr_as_ident(*expr_field.base)?;
        let member = Self::extract_member(expr_field.member)?;
        Ok(WExprField { base, member })
    }

    fn fold_right_expr_struct(&mut self, expr_struct: ExprStruct) -> Result<WExprStruct, Error> {
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
            type_path: fold_path(expr_struct.path, Some(&self.fn_folder.self_ty))?,
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
    ) -> Result<WIndexedExpr<WExprHighCall>, Error> {
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
        self.stmts.push(WMacroableStmt::Assign(WStmtAssign {
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
                ErrorType::UnsupportedConstruct("Dereference"),
                span,
            ))
        }
        syn::UnOp::Not(_) => path!(::std::ops::Not::not),
        syn::UnOp::Neg(_) => path!(::std::ops::Neg::neg),
        _ => {
            return Err(Error::new(
                ErrorType::UnsupportedConstruct("Unary operator"),
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
                ErrorType::UnsupportedConstruct("Short-circuiting AND"),
                span,
            ))
        }
        syn::BinOp::Or(_) => {
            return Err(Error::new(
                ErrorType::UnsupportedConstruct("Short-circuiting OR"),
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
                ErrorType::UnsupportedConstruct("Assignment operators"),
                span,
            ))
        }
        _ => {
            return Err(Error::new(
                ErrorType::UnsupportedConstruct("Binary operator"),
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
