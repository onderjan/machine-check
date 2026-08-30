use machine_check_common::{
    ir_common::{IrMckBinaryOp, IrMckUnaryOp, IrStdBinaryOp, IrStdUnaryOp},
    Signedness,
};
use mck::{concr::ConcreteBitvector, misc::RBound};
use syn::Lit;

use crate::{
    wir::{
        WCall, WCallArg, WExpr, WExprHighCall, WExprLowCall, WIdent, WMckBinary, WMckExt, WMckNew,
        WMckUnary, WStdBinary, WStdUnary, WType,
    },
    Error, ErrorType,
};

impl super::FnLowerer<'_> {
    pub fn lower_call(&self, call: WExprHighCall) -> Result<WExpr<WExprLowCall>, Error> {
        Ok(WExpr::Call(match call {
            WExprHighCall::Call(call) => return self.lower_normal_call(call),
            WExprHighCall::StdUnary(call) => WExprLowCall::MckUnary(self.lower_unary(call)),
            WExprHighCall::StdBinary(call) => WExprLowCall::MckBinary(self.lower_binary(call)?),
        }))
    }

    fn lower_normal_call(&self, call: WCall) -> Result<WExpr<WExprLowCall>, Error> {
        if (call
            .fn_path
            .matches_absolute(&["machine_check", "Bitvector", "new"])
            || call
                .fn_path
                .matches_absolute(&["machine_check", "Unsigned", "new"]))
            && call.args.len() == 1
        {
            if let Some(generics) = &call.fn_path.segments[1].generics {
                if generics.arguments.len() == 1 {
                    if let WCallArg::Literal(Lit::Int(lit_int)) = &call.args[0] {
                        if let Ok(value) = lit_int.base10_parse() {
                            let arg = generics.arguments[0].clone();
                            if let WType::Number(width, _span) = self.ctx.wir_type(arg) {
                                let bound = RBound::new(width);
                                let bitvector = ConcreteBitvector::new(value, bound);
                                return Ok(WExpr::Call(WExprLowCall::MckNew(WMckNew::Bitvector(
                                    bitvector,
                                ))));
                            }
                        }
                    }
                }
            }
        }

        if call
            .fn_path
            .matches_absolute(&["std", "convert", "Into", "into"])
            && call.args.len() == 1
        {
            if let WCallArg::Ident(ident) = &call.args[0] {
                // TODO: check types
                // just make into move
                return Ok(WExpr::Move(ident.clone()));
            }
        }

        if call
            .fn_path
            .matches_absolute(&["machine_check", "Ext", "ext"])
            && call.args.len() == 1
        {
            if let WCallArg::Ident(inner) = &call.args[0] {
                let inner_ty = self
                    .local_types
                    .get(inner)
                    .expect("Ext arg should have local type");
                let inner_ty = self.ctx.wir_type(inner_ty.clone());
                let mut signed = None;
                match inner_ty {
                    WType::Path(path) => {
                        if path.matches_absolute(&["machine_check", "Unsigned"]) {
                            signed = Some(false);
                        } else if path.matches_absolute(&["machine_check", "Signed"]) {
                            signed = Some(true);
                        }
                    }
                    _ => todo!("Non-path ext type"),
                }
                let Some(signed) = signed else {
                    panic!("Signedness not estabilished for extension");
                };

                if let Some(generics) = &call.fn_path.segments[1].generics {
                    if generics.arguments.len() == 1 {
                        let arg = generics.arguments[0].clone();
                        if let WType::Number(width, _span) = self.ctx.wir_type(arg) {
                            return Ok(WExpr::Call(WExprLowCall::MckExt(WMckExt {
                                signed,
                                width,
                                from: inner.clone(),
                            })));
                        }
                    }
                }

                // TODO: check types
                // just make into move
                return Ok(WExpr::Move(inner.clone()));
            }
        }

        let without_generics = call.fn_path.clone().without_generics();

        if self
            .ctx
            .definitions()
            .function_by_path(without_generics)
            .is_none()
        {
            panic!(
                "Call should be in signatures due to constraints: {:?}",
                call
            );
        }

        // must be a correct call due to the constraints placed
        Ok(WExpr::Call(WExprLowCall::Call(call)))
    }

    fn lower_unary(&self, call: WStdUnary) -> WMckUnary {
        let op = match call.op {
            IrStdUnaryOp::Not => IrMckUnaryOp::Not,
            IrStdUnaryOp::Neg => IrMckUnaryOp::Neg,
        };
        WMckUnary {
            op,
            operand: call.operand,
        }
    }

    fn lower_binary(&self, call: WStdBinary) -> Result<WMckBinary, Error> {
        let mut left_arg = call.a;
        let mut right_arg = call.b;

        let op = match call.op {
            IrStdBinaryOp::BitAnd => IrMckBinaryOp::BitAnd,
            IrStdBinaryOp::BitOr => IrMckBinaryOp::BitOr,
            IrStdBinaryOp::BitXor => IrMckBinaryOp::BitXor,
            IrStdBinaryOp::Shl => IrMckBinaryOp::LogicShl,
            IrStdBinaryOp::Shr => match self.signedness(&left_arg) {
                Some(Signedness::Signed) => IrMckBinaryOp::ArithShr,
                Some(Signedness::Unsigned) => IrMckBinaryOp::LogicShr,
                _ => {
                    return Err(Error::new(
                        ErrorType::CallConversionError("Cannot determine right shift signedness"),
                        left_arg.span(),
                    ))
                }
            },
            IrStdBinaryOp::Add => IrMckBinaryOp::Add,
            IrStdBinaryOp::Sub => IrMckBinaryOp::Sub,
            IrStdBinaryOp::Mul => IrMckBinaryOp::Mul,
            IrStdBinaryOp::Eq => IrMckBinaryOp::Eq,
            IrStdBinaryOp::Ne => IrMckBinaryOp::Ne,
            IrStdBinaryOp::Lt | IrStdBinaryOp::Le | IrStdBinaryOp::Gt | IrStdBinaryOp::Ge => {
                if matches!(call.op, IrStdBinaryOp::Gt | IrStdBinaryOp::Ge) {
                    // swap arguments
                    std::mem::swap(&mut left_arg, &mut right_arg);
                }

                let includes_equality = matches!(call.op, IrStdBinaryOp::Le | IrStdBinaryOp::Ge);

                let (Some(left_signedness), Some(right_signedness)) =
                    (self.signedness(&left_arg), self.signedness(&right_arg))
                else {
                    return Err(Error::new(
                        ErrorType::CallConversionError("Cannot determine comparison signedness"),
                        left_arg.span(),
                    ));
                };
                if left_signedness != right_signedness {
                    return Err(Error::new(
                        ErrorType::CallConversionError(
                            "Signedness of compared types does not match",
                        ),
                        left_arg.span(),
                    ));
                }

                match left_signedness {
                    Signedness::None => {
                        return Err(Error::new(
                            ErrorType::CallConversionError(
                                "Cannot compare bitvectors without signedness",
                            ),
                            left_arg.span(),
                        ))
                    }
                    Signedness::Unsigned => {
                        if includes_equality {
                            IrMckBinaryOp::Ule
                        } else {
                            IrMckBinaryOp::Ult
                        }
                    }
                    Signedness::Signed => {
                        if includes_equality {
                            IrMckBinaryOp::Sle
                        } else {
                            IrMckBinaryOp::Slt
                        }
                    }
                }
            }
            IrStdBinaryOp::Div => match self.signedness(&left_arg) {
                Some(Signedness::Signed) => IrMckBinaryOp::Sdiv,
                Some(Signedness::Unsigned) => IrMckBinaryOp::Udiv,
                _ => {
                    return Err(Error::new(
                        ErrorType::CallConversionError("Cannot determine division signedness"),
                        left_arg.span(),
                    ))
                }
            },
            IrStdBinaryOp::Rem => match self.signedness(&left_arg) {
                Some(Signedness::Signed) => IrMckBinaryOp::Srem,
                Some(Signedness::Unsigned) => IrMckBinaryOp::Urem,
                _ => {
                    return Err(Error::new(
                        ErrorType::CallConversionError("Cannot determine remainder signedness"),
                        left_arg.span(),
                    ))
                }
            },
        };

        Ok(WMckBinary {
            op,
            a: left_arg,
            b: right_arg,
        })
    }

    fn signedness(&self, ident: &WIdent) -> Option<Signedness> {
        let ty = self.local_types.get(ident);
        let Some(ty) = ty else {
            // type is not in local ident types, do not determine signedness
            return None;
        };
        let ty = self.ctx.wir_type(ty.clone());
        self.type_signedness(ty)
    }

    fn type_signedness(&self, ty: WType) -> Option<Signedness> {
        match ty {
            WType::Path(path) => {
                if path.matches_absolute(&["machine_check", "Unsigned"]) {
                    return Some(Signedness::Unsigned);
                }
                if path.matches_absolute(&["machine_check", "Signed"]) {
                    return Some(Signedness::Signed);
                }
                if path.matches_absolute(&["machine_check", "Bitvector"]) {
                    return Some(Signedness::None);
                }
                None
            }
            WType::Reference(inner, _span) => {
                let inner = self.ctx.wir_type(inner);
                self.type_signedness(inner)
            }
            WType::Number(_num, _span) => None,
        }
    }
}
