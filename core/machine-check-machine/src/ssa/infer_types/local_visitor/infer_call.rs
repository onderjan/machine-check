use crate::{
    ssa::{
        error::{DescriptionError, DescriptionErrorType},
        infer_types::is_type_fully_specified,
    },
    wir::{
        WBasicType, WCallArg, WCallFunc, WExprCall, WGeneric, WPartialGeneralType, WPath,
        WReference, WType, WTypeArray,
    },
};

impl super::LocalVisitor<'_> {
    pub(super) fn infer_call_result_type(
        &mut self,
        expr_call: &WExprCall<WCallFunc<WBasicType>>,
    ) -> WPartialGeneralType<WBasicType> {
        // discover the type based on the call function
        if let Some(ty) = skip_unknown(self.infer_init(&expr_call.fn_path.0)) {
            return ty;
        }
        if let Some(ty) = skip_unknown(self.infer_into(expr_call)) {
            return ty;
        }
        if let Some(ty) = skip_unknown(self.infer_clone(expr_call)) {
            return ty;
        }
        if let Some(ty) = skip_unknown(self.infer_array_read(expr_call)) {
            return ty;
        }
        if let Some(ty) = skip_unknown(self.infer_array_write(expr_call)) {
            return ty;
        }
        if let Some(ty) = skip_unknown(self.infer_ext(expr_call)) {
            return ty;
        }
        if let Some(ty) = skip_unknown(self.infer_return_arg_fns(expr_call)) {
            return ty;
        }
        if let Some(ty) = skip_unknown(self.infer_return_bool_fns(&expr_call.fn_path.0)) {
            return ty;
        }
        WPartialGeneralType::Unknown
    }

    fn infer_init(&mut self, fn_path: &WPath<WBasicType>) -> WPartialGeneralType<WBasicType> {
        let is_bitvector = fn_path.matches_absolute(&["machine_check", "Bitvector", "new"]);
        let is_unsigned = fn_path.matches_absolute(&["machine_check", "Unsigned", "new"]);
        let is_signed = fn_path.matches_absolute(&["machine_check", "Signed", "new"]);

        // bitvector initialization
        if is_bitvector || is_unsigned || is_signed {
            // infer bitvector-style type
            if let Some(generics) = &fn_path.segments[1].generics {
                if generics.inner.len() == 1 {
                    if let WGeneric::Const(width) = generics.inner[0] {
                        let ty = if is_bitvector {
                            WBasicType::Bitvector(width)
                        } else if is_unsigned {
                            WBasicType::Unsigned(width)
                        } else {
                            WBasicType::Signed(width)
                        };

                        return WPartialGeneralType::Normal(ty.into_type());
                    }
                }
            }
        }
        // array initialization
        if fn_path.matches_absolute(&["machine_check", "BitvectorArray", "new_filled"]) {
            // infer array type
            if let Some(generics) = &fn_path.segments[1].generics {
                if generics.inner.len() == 2 {
                    if let (WGeneric::Const(index_width), WGeneric::Const(element_width)) =
                        (&generics.inner[0], &generics.inner[1])
                    {
                        return WPartialGeneralType::Normal(
                            WBasicType::BitvectorArray(WTypeArray {
                                index_width: *index_width,
                                element_width: *element_width,
                            })
                            .into_type(),
                        );
                    }
                }
            }
        }
        WPartialGeneralType::Unknown
    }

    fn infer_into(
        &mut self,
        call: &WExprCall<WCallFunc<WBasicType>>,
    ) -> WPartialGeneralType<WBasicType> {
        // Into trait
        if !call
            .fn_path
            .0
            .matches_absolute(&["std", "convert", "Into", "into"])
        {
            return WPartialGeneralType::Unknown;
        }
        // the argument can be given
        let Some(generics) = &call.fn_path.0.segments[2].generics else {
            return WPartialGeneralType::Unknown;
        };

        if generics.inner.len() != 1 {
            self.push_error(DescriptionError::new(
                DescriptionErrorType::UnsupportedConstruct(
                    "Into without exactly one generic argument",
                ),
                call.span(),
            ));
            return WPartialGeneralType::Unknown;
        }
        let WGeneric::Type(ty) = &generics.inner[0] else {
            self.push_error(DescriptionError::new(
                DescriptionErrorType::UnsupportedConstruct("Into generic argument without a type"),
                call.span(),
            ));
            return WPartialGeneralType::Unknown;
        };

        WPartialGeneralType::Normal(ty.clone())
    }

    fn infer_clone(
        &mut self,
        call: &WExprCall<WCallFunc<WBasicType>>,
    ) -> WPartialGeneralType<WBasicType> {
        if !call
            .fn_path
            .0
            .matches_absolute(&["std", "clone", "Clone", "clone"])
        {
            return WPartialGeneralType::Unknown;
        }
        let Ok(Some(arg_type)) = self.get_normal_arg_type(call, 0, 1) else {
            return WPartialGeneralType::Unknown;
        };
        // the argument type is a reference, dereference it

        if matches!(arg_type.reference, WReference::None) {
            self.push_error(DescriptionError::new(
                DescriptionErrorType::UnsupportedConstruct(
                    "Clone first argument not being a reference",
                ),
                call.span(),
            ));
            return WPartialGeneralType::Unknown;
        }
        let mut result_type = arg_type.clone();
        result_type.reference = WReference::None;
        WPartialGeneralType::Normal(result_type)
    }

    fn infer_array_read(
        &mut self,
        call: &WExprCall<WCallFunc<WBasicType>>,
    ) -> WPartialGeneralType<WBasicType> {
        if !call
            .fn_path
            .0
            .matches_absolute(&["mck", "forward", "ReadWrite", "read"])
        {
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

    fn infer_array_write(
        &mut self,
        call: &WExprCall<WCallFunc<WBasicType>>,
    ) -> WPartialGeneralType<WBasicType> {
        if !call
            .fn_path
            .0
            .matches_absolute(&["mck", "forward", "ReadWrite", "write"])
        {
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

    fn infer_ext(
        &mut self,
        call: &WExprCall<WCallFunc<WBasicType>>,
    ) -> WPartialGeneralType<WBasicType> {
        // --- EXT ---

        if !call
            .fn_path
            .0
            .matches_absolute(&["machine_check", "Ext", "ext"])
        {
            return WPartialGeneralType::Unknown;
        }

        // find out the target width
        let Some(generics) = &call.fn_path.0.segments[1].generics else {
            return WPartialGeneralType::Unknown;
        };

        if generics.inner.len() != 1 {
            self.push_error(DescriptionError::new(
                DescriptionErrorType::UnsupportedConstruct(
                    "Ext without exactly one generic argument",
                ),
                call.span(),
            ));
            return WPartialGeneralType::Unknown;
        }
        let WGeneric::Const(width) = &generics.inner[0] else {
            self.push_error(DescriptionError::new(
                DescriptionErrorType::UnsupportedConstruct("Non-constant ext generic argument"),
                call.span(),
            ));
            return WPartialGeneralType::Unknown;
        };

        // change the width of the type in the argument

        let Ok(Some(arg_type)) = self.get_normal_arg_type(call, 0, 1) else {
            return WPartialGeneralType::Unknown;
        };

        let result = match arg_type.inner {
            WBasicType::Bitvector(_) => Some(WBasicType::Bitvector(*width)),
            WBasicType::Unsigned(_) => Some(WBasicType::Unsigned(*width)),
            WBasicType::Signed(_) => Some(WBasicType::Signed(*width)),
            _ => None,
        };
        if let Some(result) = result {
            WPartialGeneralType::Normal(result.into_type())
        } else {
            WPartialGeneralType::Unknown
        }
    }

    fn infer_return_arg_fns(
        &mut self,
        call: &WExprCall<WCallFunc<WBasicType>>,
    ) -> WPartialGeneralType<WBasicType> {
        let std_ops_fns: [(&str, &str); 12] = [
            // arithmetic
            ("Neg", "neg"),
            ("Add", "add"),
            ("Sub", "sub"),
            ("Mul", "mul"),
            ("Div", "div"),
            ("Rem", "rem"),
            // bitwise
            ("Not", "not"),
            ("BitAnd", "bitand"),
            ("BitOr", "bitor"),
            ("BitXor", "bitxor"),
            // shifts
            ("Shl", "shl"),
            ("Shr", "shr"),
        ];

        // functions that retain return type in all arguments
        for (bit_result_trait, bit_result_fn) in std_ops_fns {
            if call
                .fn_path
                .0
                .matches_absolute(&["std", "ops", bit_result_trait, bit_result_fn])
            {
                // take the type from the first argument where the type is known and inferrable
                for arg in &call.args {
                    let WCallArg::Ident(arg_ident) = arg else {
                        continue;
                    };
                    let arg_type = self.local_ident_types.get(arg_ident);
                    if let Some(arg_type) = arg_type {
                        if is_type_fully_specified(arg_type) {
                            return arg_type.clone();
                        }
                    }
                }
                return WPartialGeneralType::Unknown;
            }
        }
        WPartialGeneralType::Unknown
    }

    fn infer_return_bool_fns(
        &mut self,
        fn_path: &WPath<WBasicType>,
    ) -> WPartialGeneralType<WBasicType> {
        let std_cmp_fns: [(&str, &str); 6] = [
            ("PartialEq", "eq"),
            ("PartialEq", "ne"),
            ("PartialOrd", "lt"),
            ("PartialOrd", "le"),
            ("PartialOrd", "gt"),
            ("PartialOrd", "ge"),
        ];

        // functions that return bool
        for (bit_result_trait, bit_result_fn) in std_cmp_fns {
            if fn_path.matches_absolute(&["std", "cmp", bit_result_trait, bit_result_fn]) {
                return WPartialGeneralType::Normal(WBasicType::Boolean.into_type());
            }
        }

        WPartialGeneralType::Unknown
    }

    fn get_normal_arg_type<'a>(
        &'a mut self,
        call: &WExprCall<WCallFunc<WBasicType>>,
        arg_index: usize,
        num_args: usize,
    ) -> Result<Option<&'a WType<WBasicType>>, ()> {
        assert!(arg_index < num_args);
        if num_args != call.args.len() {
            self.push_error(DescriptionError::new(
                DescriptionErrorType::IllegalConstruct(format!(
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
