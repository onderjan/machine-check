use crate::{
    description::{Error, ErrorType},
    wir::{
        WBasicType, WCall, WCallArg, WExprHighCall, WHighMckExt, WHighMckNew, WHighStdInto,
        WHighStdIntoType, WIdent, WPartialGeneralType, WReference, WStdBinary, WStdUnary, WType,
    },
};

impl super::FnInferrer<'_> {
    pub(super) fn infer_call_result_type(
        &mut self,
        expr_call: &WExprHighCall<WBasicType>,
    ) -> WPartialGeneralType<WBasicType> {
        match expr_call {
            WExprHighCall::Call(call) => {
                // TODO: array read and write in the type system
                if let Some(result) = skip_unknown(self.infer_array_read(call)) {
                    return result;
                }
                if let Some(result) = skip_unknown(self.infer_array_write(call)) {
                    return result;
                }
                WPartialGeneralType::Unknown
            }
            WExprHighCall::StdUnary(call) => self.infer_unary(call),
            WExprHighCall::StdBinary(call) => self.infer_binary(call),
            WExprHighCall::MckExt(call) => self.infer_ext(call),
            WExprHighCall::MckNew(call) => self.infer_new(call),
            WExprHighCall::StdInto(call) => self.infer_into(call),
            WExprHighCall::StdClone(from) => self.infer_clone(from),
            //WExprHighCall::ArrayRead(call) => self.infer_array_read(call),
            //WExprHighCall::ArrayWrite(call) => self.infer_array_write(call),
        }
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

    fn infer_clone(&mut self, from: &WIdent) -> WPartialGeneralType<WBasicType> {
        let Some(WPartialGeneralType::Normal(from_type)) = self.local_ident_types.get(from) else {
            return WPartialGeneralType::Unknown;
        };
        // the argument type is a reference, dereference it

        if matches!(from_type.reference, WReference::None) {
            self.push_error(Error::new(
                ErrorType::UnsupportedConstruct("Clone first argument not being a reference"),
                from.span(),
            ));
            return WPartialGeneralType::Unknown;
        }
        let mut result_type = from_type.clone();
        result_type.reference = WReference::None;
        WPartialGeneralType::Normal(result_type)
    }

    fn infer_array_read(&mut self, call: &WCall<WBasicType>) -> WPartialGeneralType<WBasicType> {
        let fn_path = &call.fn_path;
        if !fn_path.matches_absolute(&["mck", "forward", "ReadWrite", "read"]) {
            return WPartialGeneralType::Unknown;
        }
        // infer from first argument which should be a reference to the array
        let Ok(Some(arg_type)) = self.get_normal_arg_type(call, 0, 2) else {
            return WPartialGeneralType::Unknown;
        };
        // the argument type is a reference to the array, construct the bitvector type
        if matches!(arg_type.reference, WReference::None) {
            // array read reference argument is produced internally, so this is an internal error
            panic!("First argument of array read should be a reference");
        }

        let WBasicType::BitvectorArray(type_array) = &arg_type.inner else {
            // unexpected type, do not infer
            return WPartialGeneralType::Unknown;
        };
        WPartialGeneralType::Normal(WBasicType::Bitvector(type_array.element_width).into_type())
    }

    fn infer_array_write(&mut self, call: &WCall<WBasicType>) -> WPartialGeneralType<WBasicType> {
        let fn_path = &call.fn_path;
        if !fn_path.matches_absolute(&["mck", "forward", "ReadWrite", "write"]) {
            return WPartialGeneralType::Unknown;
        }
        // infer from first argument which should be a reference to the array
        let Ok(Some(arg_type)) = self.get_normal_arg_type(call, 0, 3) else {
            return WPartialGeneralType::Unknown;
        };
        // the argument type is a reference to the array, construct the bitvector type
        if matches!(arg_type.reference, WReference::None) {
            // array write reference argument is produced internally, so this is an internal error
            panic!("First argument of array read should be a reference");
        }
        // array write returns the array, just dereferenced
        WPartialGeneralType::Normal(arg_type.inner.clone().into_type())
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

    fn get_normal_arg_type<'a>(
        &'a mut self,
        call: &WCall<WBasicType>,
        arg_index: usize,
        num_args: usize,
    ) -> Result<Option<&'a WType<WBasicType>>, ()> {
        assert!(arg_index < num_args);
        if num_args != call.args.len() {
            self.push_error(Error::new(
                ErrorType::IllegalConstruct(format!(
                    "Call must have exactly {} arguments",
                    call.args.len()
                )),
                call.span(),
            ));
            return Err(());
        }
        let arg = &call.args[arg_index];
        let WCallArg::Ident(arg_ident) = arg else {
            // TODO: this should not be a panic as it is not internal
            panic!("Call argument should be ident");
        };
        let result = self.local_ident_types.get(arg_ident);
        Ok(if let Some(WPartialGeneralType::Normal(result)) = result {
            Some(result)
        } else {
            None
        })
    }
}

fn skip_unknown(ty: WPartialGeneralType<WBasicType>) -> Option<WPartialGeneralType<WBasicType>> {
    if let WPartialGeneralType::Unknown = ty {
        None
    } else {
        Some(ty)
    }
}
