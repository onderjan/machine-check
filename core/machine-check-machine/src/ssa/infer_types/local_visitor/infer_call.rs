use crate::{
    ssa::infer_types::is_type_fully_specified,
    wir::{
        WBasicType, WCallArg, WExprCall, WGeneric, WPath, WReference, WSimpleType, WType,
        WTypeArray,
    },
    ErrorType, MachineError,
};

impl super::LocalVisitor<'_> {
    pub(super) fn infer_call_result_type(&mut self, expr_call: &WExprCall) -> Option<WType> {
        // discover the type based on the call function
        if let Some(ty) = self.infer_init(&expr_call.fn_path) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_into(expr_call) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_clone(expr_call) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_array_read(expr_call) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_array_write(expr_call) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_ext(expr_call) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_return_arg_fns(expr_call) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_return_bool_fns(&expr_call.fn_path) {
            return Some(ty);
        }
        None
    }

    fn infer_init(&mut self, fn_path: &WPath) -> Option<WType> {
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
                            WSimpleType::Basic(WBasicType::Bitvector(width))
                        } else if is_unsigned {
                            WSimpleType::Basic(WBasicType::Unsigned(width))
                        } else {
                            WSimpleType::Basic(WBasicType::Signed(width))
                        };

                        return Some(ty.into_type());
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
                        return Some(
                            WSimpleType::Basic(WBasicType::BitvectorArray(WTypeArray {
                                index_width: *index_width,
                                element_width: *element_width,
                            }))
                            .into_type(),
                        );
                    }
                }
            }
        }
        None
    }

    fn infer_into(&mut self, call: &WExprCall) -> Option<WType> {
        // Into trait
        if !call
            .fn_path
            .matches_absolute(&["std", "convert", "Into", "into"])
        {
            return None;
        }
        // the argument can be given
        let Some(generics) = &call.fn_path.segments[2].generics else {
            return None;
        };

        if generics.inner.len() != 1 {
            self.push_error(MachineError::new(
                ErrorType::UnsupportedConstruct(String::from(
                    "Into should have exactly one generic argument",
                )),
                call.span(),
            ));
            return None;
        }
        let WGeneric::Type(ty) = &generics.inner[0] else {
            self.push_error(MachineError::new(
                ErrorType::UnsupportedConstruct(String::from(
                    "Generic argument should contain a type",
                )),
                call.span(),
            ));
            return None;
        };

        Some(ty.clone().into_type())
    }

    fn infer_clone(&mut self, call: &WExprCall) -> Option<WType> {
        if !call
            .fn_path
            .matches_absolute(&["std", "clone", "Clone", "clone"])
        {
            return None;
        }
        let Ok(arg_type) = self.get_arg_type(call, 0, 1) else {
            return None;
        };
        let arg_type = arg_type?;
        // the argument type is a reference, dereference it

        if matches!(arg_type.reference, WReference::None) {
            self.push_error(MachineError::new(
                ErrorType::UnsupportedConstruct(String::from(
                    "First argument of clone should be a reference",
                )),
                call.span(),
            ));
            return None;
        }
        let mut result_type = arg_type.clone();
        result_type.reference = WReference::None;
        Some(result_type)
    }

    fn infer_array_read(&mut self, call: &WExprCall) -> Option<WType> {
        if !call
            .fn_path
            .matches_absolute(&["mck", "forward", "ReadWrite", "read"])
        {
            return None;
        }
        // infer from first argument which should be a reference to the array
        let Ok(Some(arg_type)) = self.get_arg_type(call, 0, 2) else {
            return None;
        };
        // the argument type is a reference to the array, construct the bitvector type
        if matches!(arg_type.reference, WReference::None) {
            // array read reference argument is produced internally, so this is an internal error
            panic!("First argument of array read should be a reference");
        }

        let WSimpleType::Basic(WBasicType::BitvectorArray(type_array)) = &arg_type.inner else {
            // unexpected type, do not infer
            return None;
        };
        Some(WSimpleType::Basic(WBasicType::Bitvector(type_array.element_width)).into_type())
    }

    fn infer_array_write(&mut self, call: &WExprCall) -> Option<WType> {
        if !call
            .fn_path
            .matches_absolute(&["mck", "forward", "ReadWrite", "write"])
        {
            return None;
        }
        // infer from first argument which should be a reference to the array
        let Ok(Some(arg_type)) = self.get_arg_type(call, 0, 3) else {
            return None;
        };
        // the argument type is a reference to the array, construct the bitvector type
        if matches!(arg_type.reference, WReference::None) {
            // array write reference argument is produced internally, so this is an internal error
            panic!("First argument of array read should be a reference");
        }
        // array write returns the array, just dereferenced
        Some(arg_type.inner.clone().into_type())
    }

    fn infer_ext(&mut self, call: &WExprCall) -> Option<WType> {
        // --- EXT ---

        if !call
            .fn_path
            .matches_absolute(&["machine_check", "Ext", "ext"])
        {
            return None;
        }

        // find out the target width
        let Some(generics) = &call.fn_path.segments[1].generics else {
            return None;
        };

        if generics.inner.len() != 1 {
            self.push_error(MachineError::new(
                ErrorType::UnsupportedConstruct(String::from(
                    "Ext should have exactly one generic argument",
                )),
                call.span(),
            ));
            return None;
        }
        let WGeneric::Const(width) = &generics.inner[0] else {
            self.push_error(MachineError::new(
                ErrorType::UnsupportedConstruct(String::from(
                    "Ext generic argument should contain a constant",
                )),
                call.span(),
            ));
            return None;
        };

        // change the width of the type in the argument

        let Ok(Some(arg_type)) = self.get_arg_type(call, 0, 1) else {
            return None;
        };

        match arg_type.inner {
            WSimpleType::Basic(WBasicType::Bitvector(_)) => {
                Some(WSimpleType::Basic(WBasicType::Bitvector(*width)))
            }
            WSimpleType::Basic(WBasicType::Unsigned(_)) => {
                Some(WSimpleType::Basic(WBasicType::Unsigned(*width)))
            }
            WSimpleType::Basic(WBasicType::Signed(_)) => {
                Some(WSimpleType::Basic(WBasicType::Signed(*width)))
            }
            _ => None,
        }
        .map(|simple_type| simple_type.into_type())
    }

    fn infer_return_arg_fns(&mut self, call: &WExprCall) -> Option<WType> {
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
                .matches_absolute(&["std", "ops", bit_result_trait, bit_result_fn])
            {
                // take the type from the first argument where the type is known and inferrable
                for arg in &call.args {
                    let WCallArg::Ident(arg_ident) = arg else {
                        continue;
                    };
                    let arg_type = self.local_ident_types.get(arg_ident).map(|a| a.as_ref());
                    if let Some(Some(arg_type)) = arg_type {
                        if is_type_fully_specified(arg_type) {
                            return Some(arg_type.clone());
                        }
                    }
                }
                return None;
            }
        }
        None
    }

    fn infer_return_bool_fns(&mut self, fn_path: &WPath) -> Option<WType> {
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
                return Some(WSimpleType::Basic(WBasicType::Boolean).into_type());
            }
        }

        None
    }

    fn get_arg_type<'a>(
        &'a mut self,
        call: &WExprCall,
        arg_index: usize,
        num_args: usize,
    ) -> Result<Option<&'a WType>, ()> {
        assert!(arg_index < num_args);
        if num_args != call.args.len() {
            self.push_error(MachineError::new(
                ErrorType::UnsupportedConstruct(format!(
                    "Expected {} parameters, but {} supplied",
                    num_args,
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
        Ok(self
            .local_ident_types
            .get(arg_ident)
            .and_then(|ty| ty.as_ref()))
    }
}
