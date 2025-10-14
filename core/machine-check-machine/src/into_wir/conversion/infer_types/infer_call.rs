use machine_check_common::ir_common::{IrReference, IrStdBinaryOp};

use crate::{
    into_wir::Error,
    wir::{
        WArrayRead, WArrayWrite, WBasicType, WExprHighCall, WHighMckExt, WHighMckNew, WHighStdInto,
        WHighStdIntoType, WIdent, WPartialBasicType, WPartialGeneralType, WSpanned, WStdBinary,
        WStdUnary, WType,
    },
};

impl super::FnInferrer<'_> {
    pub fn infer_call_result_type(
        &mut self,
        expr_call: &mut WExprHighCall,
        current_result_type: &WPartialGeneralType,
    ) -> Result<WPartialGeneralType, Error> {
        Ok(match expr_call {
            WExprHighCall::Call(_) => {
                // no inference for general calls yet
                WPartialGeneralType::Unknown
            }
            WExprHighCall::StdUnary(call) => self.infer_unary(call),
            WExprHighCall::StdBinary(call) => self.infer_binary(call),
            WExprHighCall::MckExt(call) => self.infer_ext(call),
            WExprHighCall::MckNew(call) => self.infer_new(call),
            WExprHighCall::BooleanNew(_) => WPartialGeneralType::Normal(WType {
                reference: IrReference::None,
                inner: WPartialBasicType::Boolean,
            }),
            WExprHighCall::StdInto(call) => self.infer_into(call, current_result_type),
            WExprHighCall::StdClone(from) => self.infer_clone(from)?,
            WExprHighCall::ArrayRead(read) => self.infer_array_read(read),
            WExprHighCall::ArrayWrite(write) => self.infer_array_write(write),
            WExprHighCall::Phi(_, _)
            | WExprHighCall::PhiTaken(_)
            | WExprHighCall::PhiNotTaken
            | WExprHighCall::PhiUninit => WPartialGeneralType::Unknown,
        })
    }

    fn infer_unary(&mut self, call: &WStdUnary) -> WPartialGeneralType {
        self.infer_same_args(&[&call.operand])
    }

    fn infer_binary(&mut self, call: &WStdBinary) -> WPartialGeneralType {
        match call.op {
            IrStdBinaryOp::BitAnd
            | IrStdBinaryOp::BitOr
            | IrStdBinaryOp::BitXor
            | IrStdBinaryOp::Shl
            | IrStdBinaryOp::Shr
            | IrStdBinaryOp::Add
            | IrStdBinaryOp::Sub
            | IrStdBinaryOp::Mul => self.infer_same_args(&[&call.a, &call.b]),
            IrStdBinaryOp::Eq
            | IrStdBinaryOp::Ne
            | IrStdBinaryOp::Lt
            | IrStdBinaryOp::Le
            | IrStdBinaryOp::Gt
            | IrStdBinaryOp::Ge => {
                // infer operands, but the result type is boolean
                let _operand_type = self.infer_same_args(&[&call.a, &call.b]);
                WPartialGeneralType::Normal(WPartialBasicType::Boolean.into_type())
            }
            IrStdBinaryOp::Div | IrStdBinaryOp::Rem => {
                // infer and convert to panic result
                let ty = self.infer_same_args(&[&call.a, &call.b]);
                if let WPartialGeneralType::Normal(ty) = ty {
                    WPartialGeneralType::PanicResult(Some(ty))
                } else {
                    WPartialGeneralType::Unknown
                }
            }
        }
    }

    fn infer_ext(&mut self, call: &WHighMckExt) -> WPartialGeneralType {
        // change the width of the type in the argument

        let Some(WPartialGeneralType::Normal(arg_type)) = self.local_ident_types.get(&call.from)
        else {
            return WPartialGeneralType::Unknown;
        };

        let result = match arg_type.inner {
            WPartialBasicType::Bitvector(_) => Some(WPartialBasicType::Bitvector(call.width)),
            WPartialBasicType::Unsigned(_) => Some(WPartialBasicType::Unsigned(call.width)),
            WPartialBasicType::Signed(_) => Some(WPartialBasicType::Signed(call.width)),
            _ => None,
        };
        if let Some(result) = result {
            WPartialGeneralType::Normal(result.into_type())
        } else {
            WPartialGeneralType::Unknown
        }
    }

    fn infer_new(&mut self, call: &WHighMckNew) -> WPartialGeneralType {
        WPartialGeneralType::Normal(
            match call {
                WHighMckNew::BitvectorArray(type_array, _) => {
                    WPartialBasicType::BitvectorArray(type_array.clone())
                }
                WHighMckNew::Bitvector(width, _) => WPartialBasicType::Bitvector(*width),
                WHighMckNew::Unsigned(width, _) => WPartialBasicType::Unsigned(*width),
                WHighMckNew::Signed(width, _) => WPartialBasicType::Signed(*width),
            }
            .into_type(),
        )
    }

    fn infer_into(
        &mut self,
        call: &mut WHighStdInto,
        current_result_type: &WPartialGeneralType,
    ) -> WPartialGeneralType {
        let arg_type = self.local_ident_types.get(&call.from);
        let mut known_width = None;
        if let Some(WPartialGeneralType::Normal(arg_type)) = arg_type {
            match arg_type.inner {
                WPartialBasicType::Bitvector(width)
                | WPartialBasicType::Unsigned(width)
                | WPartialBasicType::Signed(width) => {
                    known_width = width;
                }
                _ => {}
            }
        }

        eprintln!(
            "Inferring into: {:?}, current result type: {:?}",
            call, current_result_type
        );
        WPartialGeneralType::Normal(
            match &mut call.ty {
                WHighStdIntoType::Bitvector(call_width) => {
                    if call_width.is_none() {
                        if let Some(known_width) = known_width {
                            *call_width = Some(known_width);
                        }
                    }

                    WPartialBasicType::Bitvector(*call_width)
                }
                WHighStdIntoType::Unsigned(call_width) => {
                    if call_width.is_none() {
                        if let Some(known_width) = known_width {
                            *call_width = Some(known_width);
                        }
                    }

                    WPartialBasicType::Unsigned(*call_width)
                }
                WHighStdIntoType::Signed(call_width) => {
                    if call_width.is_none() {
                        if let Some(known_width) = known_width {
                            *call_width = Some(known_width);
                        }
                    }

                    WPartialBasicType::Signed(*call_width)
                }
            }
            .into_type(),
        )
    }

    fn infer_clone(&mut self, from: &WIdent) -> Result<WPartialGeneralType, Error> {
        let Some(WPartialGeneralType::Normal(from_type)) = self.local_ident_types.get(from) else {
            return Ok(WPartialGeneralType::Unknown);
        };
        // the argument type is a reference, dereference it

        if matches!(from_type.reference, IrReference::None) {
            return Err(Error::unsupported_construct(
                "Clone first argument not being a reference",
                from.wir_span(),
            ));
        }
        let mut result_type = from_type.clone();
        result_type.reference = IrReference::None;
        Ok(WPartialGeneralType::Normal(result_type))
    }

    fn infer_array_read(&mut self, read: &WArrayRead) -> WPartialGeneralType {
        // infer from the reference to the array
        let Some(WPartialGeneralType::Normal(array_type)) = self.local_ident_types.get(&read.base)
        else {
            return WPartialGeneralType::Unknown;
        };
        // the argument type is a reference to the array, construct the bitvector type
        if matches!(array_type.reference, IrReference::None) {
            // array read reference argument is produced internally, so this is an internal error
            panic!("First argument of array read should be a reference");
        }

        let WPartialBasicType::BitvectorArray(array_type) = &array_type.inner else {
            // unexpected type, do not infer
            return WPartialGeneralType::Unknown;
        };
        WPartialGeneralType::Normal(
            WPartialBasicType::Bitvector(Some(array_type.element_width)).into_type(),
        )
    }

    fn infer_array_write(&mut self, write: &WArrayWrite) -> WPartialGeneralType {
        // infer from the reference to the array
        let Some(WPartialGeneralType::Normal(array_type)) = self.local_ident_types.get(&write.base)
        else {
            return WPartialGeneralType::Unknown;
        };
        // the argument type is a reference to the array, construct the bitvector type
        if matches!(array_type.reference, IrReference::None) {
            // array write reference argument is produced internally, so this is an internal error
            panic!("First argument of array read should be a reference");
        }
        // array write returns the array, just dereferenced
        WPartialGeneralType::Normal(array_type.inner.clone().into_type())
    }

    fn infer_same_args(&mut self, args: &[&WIdent]) -> WPartialGeneralType {
        // take the type from the first argument where the type is known and inferrable
        let mut each_arg_type = None;
        for arg in args {
            let arg_type = self.local_ident_types.get(arg);
            if let Some(arg_type) = arg_type {
                if arg_type.is_fully_determined() {
                    each_arg_type = Some(arg_type.clone());
                    break;
                }
            }
        }

        let Some(each_arg_type) = each_arg_type else {
            return WPartialGeneralType::Unknown;
        };

        for arg in args {
            /*println!(
                "Inferred same arg type {:?} for arg {:?}, which is currently {:?}",
                each_arg_type,
                arg,
                self.local_ident_types.get(arg)
            );*/
            if self
                .local_ident_types
                .get(arg)
                .is_none_or(|arg_type| !arg_type.is_fully_determined())
            {
                self.local_ident_types
                    .insert((*arg).clone(), each_arg_type.clone());
            }
        }

        each_arg_type
    }
}
