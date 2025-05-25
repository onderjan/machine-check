use crate::{
    description::{Error, ErrorType},
    wir::{
        WArrayRead, WArrayWrite, WBasicType, WExprHighCall, WHighMckExt, WHighMckNew, WHighStdInto,
        WHighStdIntoType, WIdent, WPartialGeneralType, WReference, WStdBinary, WStdUnary,
    },
};

impl super::FnInferrer<'_> {
    pub fn infer_call_result_type(
        &mut self,
        expr_call: &WExprHighCall,
    ) -> Result<WPartialGeneralType<WBasicType>, Error> {
        Ok(match expr_call {
            WExprHighCall::Call(_) => {
                // no inference for general calls yet
                WPartialGeneralType::Unknown
            }
            WExprHighCall::StdUnary(call) => self.infer_unary(call),
            WExprHighCall::StdBinary(call) => self.infer_binary(call),
            WExprHighCall::MckExt(call) => self.infer_ext(call),
            WExprHighCall::MckNew(call) => self.infer_new(call),
            WExprHighCall::StdInto(call) => self.infer_into(call),
            WExprHighCall::StdClone(from) => self.infer_clone(from)?,
            WExprHighCall::ArrayRead(read) => self.infer_array_read(read),
            WExprHighCall::ArrayWrite(write) => self.infer_array_write(write),
            WExprHighCall::Phi(_, _)
            | WExprHighCall::PhiTaken(_)
            | WExprHighCall::PhiNotTaken
            | WExprHighCall::PhiUninit => WPartialGeneralType::Unknown,
        })
    }

    fn infer_unary(&mut self, call: &WStdUnary) -> WPartialGeneralType<WBasicType> {
        self.infer_same_args(&[&call.operand])
    }

    fn infer_binary(&mut self, call: &WStdBinary) -> WPartialGeneralType<WBasicType> {
        match call.op {
            crate::wir::WStdBinaryOp::BitAnd
            | crate::wir::WStdBinaryOp::BitOr
            | crate::wir::WStdBinaryOp::BitXor
            | crate::wir::WStdBinaryOp::Shl
            | crate::wir::WStdBinaryOp::Shr
            | crate::wir::WStdBinaryOp::Add
            | crate::wir::WStdBinaryOp::Sub
            | crate::wir::WStdBinaryOp::Mul => self.infer_same_args(&[&call.a, &call.b]),
            crate::wir::WStdBinaryOp::Eq
            | crate::wir::WStdBinaryOp::Ne
            | crate::wir::WStdBinaryOp::Lt
            | crate::wir::WStdBinaryOp::Le
            | crate::wir::WStdBinaryOp::Gt
            | crate::wir::WStdBinaryOp::Ge => {
                WPartialGeneralType::Normal(WBasicType::Boolean.into_type())
            }
            crate::wir::WStdBinaryOp::Div | crate::wir::WStdBinaryOp::Rem => {
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

    fn infer_ext(&mut self, call: &WHighMckExt) -> WPartialGeneralType<WBasicType> {
        // change the width of the type in the argument

        let Some(WPartialGeneralType::Normal(arg_type)) = self.local_ident_types.get(&call.from)
        else {
            return WPartialGeneralType::Unknown;
        };

        let result = match arg_type.inner {
            WBasicType::Bitvector(_) => Some(WBasicType::Bitvector(call.width)),
            WBasicType::Unsigned(_) => Some(WBasicType::Unsigned(call.width)),
            WBasicType::Signed(_) => Some(WBasicType::Signed(call.width)),
            _ => None,
        };
        if let Some(result) = result {
            WPartialGeneralType::Normal(result.into_type())
        } else {
            WPartialGeneralType::Unknown
        }
    }

    fn infer_new(&mut self, call: &WHighMckNew) -> WPartialGeneralType<WBasicType> {
        WPartialGeneralType::Normal(
            match call {
                WHighMckNew::BitvectorArray(type_array, _) => {
                    WBasicType::BitvectorArray(type_array.clone())
                }
                WHighMckNew::Bitvector(width, _) => WBasicType::Bitvector(*width),
                WHighMckNew::Unsigned(width, _) => WBasicType::Unsigned(*width),
                WHighMckNew::Signed(width, _) => WBasicType::Signed(*width),
            }
            .into_type(),
        )
    }

    fn infer_into(&mut self, call: &WHighStdInto) -> WPartialGeneralType<WBasicType> {
        WPartialGeneralType::Normal(
            match call.ty {
                WHighStdIntoType::Bitvector(width) => WBasicType::Bitvector(width),
                WHighStdIntoType::Unsigned(width) => WBasicType::Unsigned(width),
                WHighStdIntoType::Signed(width) => WBasicType::Signed(width),
            }
            .into_type(),
        )
    }

    fn infer_clone(&mut self, from: &WIdent) -> Result<WPartialGeneralType<WBasicType>, Error> {
        let Some(WPartialGeneralType::Normal(from_type)) = self.local_ident_types.get(from) else {
            return Ok(WPartialGeneralType::Unknown);
        };
        // the argument type is a reference, dereference it

        if matches!(from_type.reference, WReference::None) {
            return Err(Error::new(
                ErrorType::UnsupportedConstruct("Clone first argument not being a reference"),
                from.span(),
            ));
        }
        let mut result_type = from_type.clone();
        result_type.reference = WReference::None;
        Ok(WPartialGeneralType::Normal(result_type))
    }

    fn infer_array_read(&mut self, read: &WArrayRead) -> WPartialGeneralType<WBasicType> {
        // infer from the reference to the array
        let Some(WPartialGeneralType::Normal(array_type)) = self.local_ident_types.get(&read.base)
        else {
            return WPartialGeneralType::Unknown;
        };
        // the argument type is a reference to the array, construct the bitvector type
        if matches!(array_type.reference, WReference::None) {
            // array read reference argument is produced internally, so this is an internal error
            panic!("First argument of array read should be a reference");
        }

        let WBasicType::BitvectorArray(array_type) = &array_type.inner else {
            // unexpected type, do not infer
            return WPartialGeneralType::Unknown;
        };
        WPartialGeneralType::Normal(WBasicType::Bitvector(array_type.element_width).into_type())
    }

    fn infer_array_write(&mut self, write: &WArrayWrite) -> WPartialGeneralType<WBasicType> {
        // infer from the reference to the array
        let Some(WPartialGeneralType::Normal(array_type)) = self.local_ident_types.get(&write.base)
        else {
            return WPartialGeneralType::Unknown;
        };
        // the argument type is a reference to the array, construct the bitvector type
        if matches!(array_type.reference, WReference::None) {
            // array write reference argument is produced internally, so this is an internal error
            panic!("First argument of array read should be a reference");
        }
        // array write returns the array, just dereferenced
        WPartialGeneralType::Normal(array_type.inner.clone().into_type())
    }

    fn infer_same_args(&mut self, args: &[&WIdent]) -> WPartialGeneralType<WBasicType> {
        // take the type from the first argument where the type is known and inferrable
        for arg in args {
            let arg_type = self.local_ident_types.get(arg);
            if let Some(arg_type) = arg_type {
                if arg_type.is_fully_determined() {
                    return arg_type.clone();
                }
            }
        }

        WPartialGeneralType::Unknown
    }
}
