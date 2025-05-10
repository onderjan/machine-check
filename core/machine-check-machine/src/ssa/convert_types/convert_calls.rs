use std::collections::BTreeMap;

use crate::{
    wir::{WBasicType, WCallArg, WElementaryType, WExpr, WExprCall, WGeneralType, WIdent, WPath},
    ErrorType, MachineError,
};

use super::path_start_to_mck_forward;

impl super::Converter {
    pub fn convert_call_fn_path(
        &mut self,
        call: WExprCall<WBasicType>,
        local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
    ) -> Result<WExpr<WElementaryType>, MachineError> {
        let fn_path = call.fn_path.clone();

        let orig_call = call.clone();

        let mut call = WExprCall {
            fn_path: self.convert_basic_path(call.fn_path),
            args: call.args,
        };

        if let Some(result) = self.convert_bitwise(&fn_path) {
            call.fn_path = result;
            return Ok(WExpr::Call(call));
        }

        if let Some(result) = self.convert_arith(&fn_path) {
            call.fn_path = result;
            return Ok(WExpr::Call(call));
        }

        if let Some(result) = self.convert_eq(&fn_path) {
            call.fn_path = result;
            return Ok(WExpr::Call(call));
        }

        if let Some(result) = self.convert_cmp(&fn_path, &mut call.args, local_types) {
            call.fn_path = result?;
            return Ok(WExpr::Call(call));
        }

        if let Some(result) = self.convert_shift(&fn_path, &mut call.args, local_types) {
            call.fn_path = result?;
            return Ok(WExpr::Call(call));
        }

        if let Some(result) = self.convert_ext(&fn_path, &call.args, local_types) {
            call.fn_path = result?;
            return Ok(WExpr::Call(call));
        }

        if let Some(result) = self.convert_into(orig_call, local_types) {
            return result;
        }

        Ok(WExpr::Call(call))
    }

    fn convert_bitwise(&mut self, fn_path: &WPath<WBasicType>) -> Option<WPath<WElementaryType>> {
        let is_bit_not = fn_path.matches_absolute(&["std", "ops", "Not", "not"]);
        let is_bit_and = fn_path.matches_absolute(&["std", "ops", "BitAnd", "bitand"]);
        let is_bit_or = fn_path.matches_absolute(&["std", "ops", "BitOr", "bitor"]);
        let is_bit_xor = fn_path.matches_absolute(&["std", "ops", "BitXor", "bitxor"]);

        if !is_bit_not && !is_bit_and && !is_bit_or && !is_bit_xor {
            return None;
        }

        let mut fn_path = fn_path.clone();

        // update

        fn_path.segments[0].ident.name = String::from("mck");
        fn_path.segments[1].ident.name = String::from("forward");
        fn_path.segments[2].ident.name = String::from("Bitwise");

        // --- Bitwise ---
        if is_bit_not {
            fn_path.segments[3].ident.name = String::from("bit_not");
        }
        if is_bit_and {
            fn_path.segments[3].ident.name = String::from("bit_and");
        }
        if is_bit_or {
            fn_path.segments[3].ident.name = String::from("bit_or");
        }
        if is_bit_xor {
            fn_path.segments[3].ident.name = String::from("bit_xor");
        }
        Some(self.convert_basic_path(fn_path))
    }

    fn convert_arith(&mut self, fn_path: &WPath<WBasicType>) -> Option<WPath<WElementaryType>> {
        if fn_path.matches_absolute(&["std", "ops", "Neg", "neg"]) {
            let mut fn_path = fn_path.clone();
            fn_path.segments[0].ident.name = String::from("mck");
            fn_path.segments[1].ident.name = String::from("forward");
            fn_path.segments[2].ident.name = String::from("HwArith");
            fn_path.segments[3].ident.name = String::from("arith_neg");
            return Some(self.convert_basic_path(fn_path));
        }

        if fn_path.matches_absolute(&["std", "ops", "Add", "add"])
            || fn_path.matches_absolute(&["std", "ops", "Sub", "sub"])
            || fn_path.matches_absolute(&["std", "ops", "Mul", "mul"])
        {
            let mut fn_path = fn_path.clone();
            fn_path.segments[0].ident.name = String::from("mck");
            fn_path.segments[1].ident.name = String::from("forward");
            fn_path.segments[2].ident.name = String::from("HwArith");
            // leave the last segment as-is
            return Some(self.convert_basic_path(fn_path));
        }

        // TODO: div, rem depending on Signed/Unsigned
        None
    }

    fn convert_eq(&mut self, fn_path: &WPath<WBasicType>) -> Option<WPath<WElementaryType>> {
        if fn_path.matches_absolute(&["std", "cmp", "PartialEq", "eq"])
            || fn_path.matches_absolute(&["std", "cmp", "PartialEq", "ne"])
        {
            let mut fn_path = fn_path.clone();
            fn_path.segments[0].ident.name = String::from("mck");
            fn_path.segments[1].ident.name = String::from("forward");
            fn_path.segments[2].ident.name = String::from("TypedEq");
            // leave the last segment as-is
            return Some(self.convert_basic_path(fn_path));
        }
        None
    }

    fn convert_cmp(
        &mut self,
        fn_path: &WPath<WBasicType>,
        args: &mut [WCallArg],
        local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
    ) -> Option<Result<WPath<WElementaryType>, MachineError>> {
        if !fn_path.matches_absolute(&["std", "cmp", "PartialOrd", "lt"])
            && !fn_path.matches_absolute(&["std", "cmp", "PartialOrd", "le"])
            && !fn_path.matches_absolute(&["std", "cmp", "PartialOrd", "gt"])
            && !fn_path.matches_absolute(&["std", "cmp", "PartialOrd", "ge"])
        {
            return None;
        }
        let mut fn_path = fn_path.clone();
        fn_path.segments[0].ident.name = String::from("mck");
        fn_path.segments[1].ident.name = String::from("forward");
        fn_path.segments[2].ident.name = String::from("TypedCmp");

        // need to know type signedness
        if args.len() != 2 {
            return Some(Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Comparison should have exactly two arguments",
                )),
                fn_path.span(),
            )));
        }

        let (Some(left_is_signed), Some(right_is_signed)) = (
            signedness(&args[0], local_types),
            signedness(&args[1], local_types),
        ) else {
            return Some(Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Cannot determine comparison signedness",
                )),
                fn_path.span(),
            )));
        };
        if left_is_signed != right_is_signed {
            return Some(Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Signedness of compared types does not match",
                )),
                fn_path.span(),
            )));
        }

        let fn_prefix = if left_is_signed { "s" } else { "u" };

        // strength of inequality is preserved when arguments are swapped
        // i.e. a >= b becomes b <= a, a > b becomes b < a
        let (fn_suffix, swap_args) = match fn_path.segments[3].ident.name.as_str() {
            "lt" => ("lt", false),
            "le" => ("le", false),
            "gt" => ("lt", true),
            "ge" => ("le", true),
            _ => panic!("Unexpected comparison function"),
        };
        if swap_args {
            args.swap(0, 1);
        }

        fn_path.segments[3].ident.name = format!("{}{}", fn_prefix, fn_suffix);

        Some(Ok(self.convert_basic_path(fn_path)))
    }

    fn convert_shift(
        &mut self,
        fn_path: &WPath<WBasicType>,
        args: &mut [WCallArg],
        local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
    ) -> Option<Result<WPath<WElementaryType>, MachineError>> {
        // --- Shl ---
        if fn_path.matches_absolute(&["std", "ops", "Shl", "shl"]) {
            let mut fn_path = fn_path.clone();
            fn_path.segments[0].ident.name = String::from("mck");
            fn_path.segments[1].ident.name = String::from("forward");
            fn_path.segments[2].ident.name = String::from("HwShift");
            fn_path.segments[3].ident.name = String::from("logic_shl");
            return Some(Ok(self.convert_basic_path(fn_path)));
        }

        // --- Shr ---
        if fn_path.matches_absolute(&["std", "ops", "Shr", "shr"]) {
            let mut fn_path = fn_path.clone();
            fn_path.segments[0].ident.name = String::from("mck");
            fn_path.segments[1].ident.name = String::from("forward");
            fn_path.segments[2].ident.name = String::from("HwShift");

            // determine signedness from the first argument
            // note that in Rust, type inference depends on whether Shr is an operation or a call
            // but this should not impact our simple bitvector-signed-unsigned types
            if args.len() != 2 {
                return Some(Err(MachineError::new(
                    ErrorType::ConcreteConversionError(String::from(
                        "Right shift should have exactly two arguments",
                    )),
                    fn_path.span(),
                )));
            }

            let Some(is_signed) = signedness(&args[0], local_types) else {
                return Some(Err(MachineError::new(
                    ErrorType::ConcreteConversionError(String::from(
                        "Cannot determine right shift signedness",
                    )),
                    fn_path.span(),
                )));
            };

            let fn_name = if is_signed { "arith_shr" } else { "logic_shr" };
            fn_path.segments[3].ident.name = String::from(fn_name);
            return Some(Ok(self.convert_basic_path(fn_path)));
        }
        None
    }

    fn convert_ext(
        &mut self,
        fn_path: &WPath<WBasicType>,
        args: &[WCallArg],
        local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
    ) -> Option<Result<WPath<WElementaryType>, MachineError>> {
        if !fn_path.matches_absolute(&["machine_check", "Ext", "ext"]) {
            return None;
        }

        // need to know type signedness
        if args.len() != 1 {
            return Some(Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Bit extension should have exactly one argument",
                )),
                fn_path.span(),
            )));
        }
        let mut fn_path = path_start_to_mck_forward(fn_path.clone());

        let Some(is_signed) = signedness(&args[0], local_types) else {
            return Some(Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Cannot determine bit extension signedness",
                )),
                fn_path.span(),
            )));
        };

        let fn_name = if is_signed { "sext" } else { "uext" };
        fn_path.segments[3].ident.name = String::from(fn_name);
        Some(Ok(self.convert_basic_path(fn_path)))
    }

    fn convert_into(
        &mut self,
        call: WExprCall<WBasicType>,
        local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
    ) -> Option<Result<WExpr<WElementaryType>, MachineError>> {
        if !call
            .fn_path
            .matches_absolute(&["std", "convert", "Into", "into"])
        {
            return None;
        }
        // make sure the argument is a bitvector-related type
        // we do not need to check generics as these will be handled by Rust
        if call.args.len() != 1 {
            return Some(Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Into should have exactly one argument",
                )),
                call.fn_path.span(),
            )));
        }

        let arg_ident = match &call.args[0] {
            WCallArg::Ident(arg_ident) => {
                if let Some(arg_type) = local_types.get(arg_ident) {
                    match arg_type {
                        WGeneralType::Normal(wtype) => match wtype.inner {
                            WBasicType::Bitvector(_)
                            | WBasicType::Unsigned(_)
                            | WBasicType::Signed(_) => Some(arg_ident.clone()),
                            _ => None,
                        },
                        _ => None,
                    }
                } else {
                    None
                }
            }
            WCallArg::Literal(_) => None,
        };

        let Some(arg_ident) = arg_ident else {
            return Some(Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Unable to assure that argument to Into is a machine-check bitvector",
                )),
                call.fn_path.span(),
            )));
        };
        // into is no-op for our converted types, so change to a move
        Some(Ok(WExpr::Move(arg_ident)))
    }
}

fn signedness(
    call_arg: &WCallArg,
    local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
) -> Option<bool> {
    let WCallArg::Ident(ident) = call_arg else {
        return None;
    };

    let ty = local_types.get(ident);
    let Some(ty) = ty else {
        // type is not in local ident types, do not determine signedness
        return None;
    };
    match ty {
        WGeneralType::Normal(ty) => match ty.inner {
            WBasicType::Unsigned(_) => Some(false),
            WBasicType::Signed(_) => Some(true),
            _ => None,
        },
        _ => None,
    }
}
