use machine_check_common::{
    ir_common::{IrMckBinaryOp, IrMckUnaryOp, IrStdBinaryOp, IrStdUnaryOp},
    Signedness,
};
use mck::{concr::ConcreteBitvector, misc::RBound};
use syn::Lit;

use crate::{
    into_wir::{conversion::convert_types::FnTypeConverter, Error, ErrorType},
    wir::{
        WCall, WCallArg, WExpr, WExprHighCall, WExprLowCall, WIdent, WMckBinary, WMckNew,
        WMckUnary, WPartialArgument, WSpanned, WStdBinary, WStdUnary, WType,
    },
};

impl FnTypeConverter<'_> {
    pub fn convert_call(&self, call: WExprHighCall) -> Result<WExpr<WExprLowCall>, Error> {
        Ok(WExpr::Call(match call {
            WExprHighCall::Call(call) => return self.convert_normal_call(call),
            WExprHighCall::StdUnary(call) => WExprLowCall::MckUnary(self.convert_unary(call)),
            WExprHighCall::StdBinary(call) => WExprLowCall::MckBinary(self.convert_binary(call)?),
        }))
    }

    fn convert_normal_call(&self, call: WCall) -> Result<WExpr<WExprLowCall>, Error> {
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
                            if let WPartialArgument::Uint(width, _span) = &generics.arguments[0] {
                                let bound = RBound::new(*width);
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

        todo!("Lower call {:?}", call);
    }

    fn convert_unary(&self, call: WStdUnary) -> WMckUnary {
        let op = match call.op {
            IrStdUnaryOp::Not => IrMckUnaryOp::Not,
            IrStdUnaryOp::Neg => IrMckUnaryOp::Neg,
        };
        WMckUnary {
            op,
            operand: call.operand,
        }
    }

    fn convert_binary(&self, call: WStdBinary) -> Result<WMckBinary, Error> {
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
                        left_arg.wir_span(),
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
                        left_arg.wir_span(),
                    ));
                };
                if left_signedness != right_signedness {
                    return Err(Error::new(
                        ErrorType::CallConversionError(
                            "Signedness of compared types does not match",
                        ),
                        left_arg.wir_span(),
                    ));
                }

                match left_signedness {
                    Signedness::None => {
                        return Err(Error::new(
                            ErrorType::CallConversionError(
                                "Cannot compare bitvectors without signedness",
                            ),
                            left_arg.wir_span(),
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
                        left_arg.wir_span(),
                    ))
                }
            },
            IrStdBinaryOp::Rem => match self.signedness(&left_arg) {
                Some(Signedness::Signed) => IrMckBinaryOp::Srem,
                Some(Signedness::Unsigned) => IrMckBinaryOp::Urem,
                _ => {
                    return Err(Error::new(
                        ErrorType::CallConversionError("Cannot determine remainder signedness"),
                        left_arg.wir_span(),
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

    /*
    fn convert_ext(
        call: WHighMckExt,
        local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
    ) -> Result<WMckExt, Error> {
        let signed = match signedness(&call.from, local_types) {
            Some(Signedness::Signed) => true,
            Some(Signedness::Unsigned) => false,
            _ => {
                return Err(Error::new(
                    ErrorType::CallConversionError("Cannot determine bit extension signedness"),
                    call.from.wir_span(),
                ))
            }
        };

        Ok(WMckExt {
            signed,
            width: call.width.expect("Cannot determine bit extension width"),
            from: call.from,
        })
    }

    fn convert_mck_new(call: WHighMckNew) -> Result<WMckNew, Error> {
        Ok(match call {
            WHighMckNew::Bitvector(signedness, width, constant) => {
                let width = width.expect("Created width should be known");

                fn outside_bounds_fn<T: Display>(err: mck::concr::OutsideBound<T>) -> Error {
                    Error::new(
                        ErrorType::IllegalConstruct(err.to_string()),
                        WSpan::call_site(),
                    )
                }

                WMckNew::Bitvector(match signedness {
                    Signedness::None | Signedness::Unsigned => {
                        let Ok(constant) = constant.try_into() else {
                            return Err(Error {
                                ty: ErrorType::IllegalConstruct(String::from(
                                    "Constant does not fit into u64",
                                )),
                                span: WSpan::call_site(),
                            });
                        };

                        match mck::concr::ConcreteBitvector::try_new(
                            constant,
                            mck::misc::RBound::new(width),
                        ) {
                            Ok(ok) => ok,
                            Err(err) => return Err(outside_bounds_fn(err)),
                        }
                    }
                    Signedness::Signed => {
                        let Ok(constant) = constant.try_into() else {
                            return Err(Error {
                                ty: ErrorType::IllegalConstruct(String::from(
                                    "Constant does not fit into i64",
                                )),
                                span: WSpan::call_site(),
                            });
                        };

                        match mck::concr::SignedBitvector::try_new(
                            constant,
                            mck::misc::RBound::new(width),
                        ) {
                            Ok(signed) => signed.cast_bitvector(),
                            Err(err) => return Err(outside_bounds_fn(err)),
                        }
                    }
                })
            }
            WHighMckNew::BitvectorArray(type_array, fill_element) => {
                WMckNew::BitvectorArray(type_array, fill_element)
            }
        })
    }*/

    fn signedness(&self, ident: &WIdent) -> Option<Signedness> {
        let ty = self.local_types.get(ident);
        let Some(ty) = ty else {
            // type is not in local ident types, do not determine signedness
            return None;
        };
        let ty = self.ctx.wir_type(ty.clone());
        type_signedness(ty)
    }
}

fn type_signedness(ty: WType) -> Option<Signedness> {
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
        WType::Reference(inner) => type_signedness(*inner),
    }
}
