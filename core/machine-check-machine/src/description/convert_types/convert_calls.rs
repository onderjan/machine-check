use std::collections::BTreeMap;

use crate::{
    description::{Error, ErrorType},
    wir::{
        WBasicType, WCall, WCallArg, WElementaryType, WExpr, WExprCall, WExprHighCall,
        WGeneralType, WIdent, WPath,
    },
};

use super::{convert_basic_path, path_start_to_mck_forward};

pub fn convert_call_fn_path(
    call: WExprHighCall<WBasicType>,
    local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
) -> Result<WExpr<WElementaryType, WExprCall<WElementaryType>>, Error> {
    let orig_call = call.clone();
    let WExprHighCall::Call(WCall { fn_path, args }) = call;

    let mut call = WCall {
        fn_path: convert_basic_path(fn_path.clone()),
        args,
    };

    if let Some(result) = convert_bitwise(&fn_path) {
        call.fn_path = result;
        return Ok(WExpr::Call(WExprCall::Call(call)));
    }

    if let Some(result) = convert_arith(&fn_path) {
        call.fn_path = result;
        return Ok(WExpr::Call(WExprCall::Call(call)));
    }

    if let Some(result) = convert_eq(&fn_path) {
        call.fn_path = result;
        return Ok(WExpr::Call(WExprCall::Call(call)));
    }

    if let Some(result) = convert_cmp(&fn_path, &mut call.args, local_types) {
        call.fn_path = result?;
        return Ok(WExpr::Call(WExprCall::Call(call)));
    }

    if let Some(result) = convert_shift(&fn_path, &mut call.args, local_types) {
        call.fn_path = result?;
        return Ok(WExpr::Call(WExprCall::Call(call)));
    }

    if let Some(result) = convert_ext(&fn_path, &call.args, local_types) {
        call.fn_path = result?;
        return Ok(WExpr::Call(WExprCall::Call(call)));
    }

    if let Some(result) = convert_into(orig_call, local_types) {
        return result;
    }

    Ok(WExpr::Call(WExprCall::Call(call)))
}

fn convert_bitwise(fn_path: &WPath<WBasicType>) -> Option<WPath<WElementaryType>> {
    let is_bit_not = fn_path.matches_absolute(&["std", "ops", "Not", "not"]);
    let is_bit_and = fn_path.matches_absolute(&["std", "ops", "BitAnd", "bitand"]);
    let is_bit_or = fn_path.matches_absolute(&["std", "ops", "BitOr", "bitor"]);
    let is_bit_xor = fn_path.matches_absolute(&["std", "ops", "BitXor", "bitxor"]);

    if !is_bit_not && !is_bit_and && !is_bit_or && !is_bit_xor {
        return None;
    }

    let mut fn_path = fn_path.clone();

    // update

    fn_path.segments[0].ident.set_name(String::from("mck"));
    fn_path.segments[1].ident.set_name(String::from("forward"));
    fn_path.segments[2].ident.set_name(String::from("Bitwise"));

    // --- Bitwise ---
    if is_bit_not {
        fn_path.segments[3].ident.set_name(String::from("bit_not"));
    }
    if is_bit_and {
        fn_path.segments[3].ident.set_name(String::from("bit_and"));
    }
    if is_bit_or {
        fn_path.segments[3].ident.set_name(String::from("bit_or"));
    }
    if is_bit_xor {
        fn_path.segments[3].ident.set_name(String::from("bit_xor"));
    }
    Some(convert_basic_path(fn_path))
}

fn convert_arith(fn_path: &WPath<WBasicType>) -> Option<WPath<WElementaryType>> {
    if fn_path.matches_absolute(&["std", "ops", "Neg", "neg"]) {
        let mut fn_path = fn_path.clone();
        fn_path.segments[0].ident.set_name(String::from("mck"));
        fn_path.segments[1].ident.set_name(String::from("forward"));
        fn_path.segments[2].ident.set_name(String::from("HwArith"));
        fn_path.segments[3]
            .ident
            .set_name(String::from("arith_neg"));
        return Some(convert_basic_path(fn_path));
    }

    if fn_path.matches_absolute(&["std", "ops", "Add", "add"])
        || fn_path.matches_absolute(&["std", "ops", "Sub", "sub"])
        || fn_path.matches_absolute(&["std", "ops", "Mul", "mul"])
    {
        let mut fn_path = fn_path.clone();
        fn_path.segments[0].ident.set_name(String::from("mck"));
        fn_path.segments[1].ident.set_name(String::from("forward"));
        fn_path.segments[2].ident.set_name(String::from("HwArith"));
        // leave the last segment as-is
        return Some(convert_basic_path(fn_path));
    }

    // TODO: div, rem depending on Signed/Unsigned
    None
}

fn convert_eq(fn_path: &WPath<WBasicType>) -> Option<WPath<WElementaryType>> {
    if fn_path.matches_absolute(&["std", "cmp", "PartialEq", "eq"])
        || fn_path.matches_absolute(&["std", "cmp", "PartialEq", "ne"])
    {
        let mut fn_path = fn_path.clone();
        fn_path.segments[0].ident.set_name(String::from("mck"));
        fn_path.segments[1].ident.set_name(String::from("forward"));
        fn_path.segments[2].ident.set_name(String::from("TypedEq"));
        // leave the last segment as-is
        return Some(convert_basic_path(fn_path));
    }
    None
}

fn convert_cmp(
    fn_path: &WPath<WBasicType>,
    args: &mut [WCallArg],
    local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
) -> Option<Result<WPath<WElementaryType>, Error>> {
    if !fn_path.matches_absolute(&["std", "cmp", "PartialOrd", "lt"])
        && !fn_path.matches_absolute(&["std", "cmp", "PartialOrd", "le"])
        && !fn_path.matches_absolute(&["std", "cmp", "PartialOrd", "gt"])
        && !fn_path.matches_absolute(&["std", "cmp", "PartialOrd", "ge"])
    {
        return None;
    }
    let mut fn_path = fn_path.clone();
    fn_path.segments[0].ident.set_name(String::from("mck"));
    fn_path.segments[1].ident.set_name(String::from("forward"));
    fn_path.segments[2].ident.set_name(String::from("TypedCmp"));

    // need to know type signedness
    if args.len() != 2 {
        return Some(Err(Error::new(
            ErrorType::TypeConversionError("Comparison should have exactly two arguments"),
            fn_path.span(),
        )));
    }

    let (Some(left_is_signed), Some(right_is_signed)) = (
        signedness(&args[0], local_types),
        signedness(&args[1], local_types),
    ) else {
        return Some(Err(Error::new(
            ErrorType::TypeConversionError("Cannot determine comparison signedness"),
            fn_path.span(),
        )));
    };
    if left_is_signed != right_is_signed {
        return Some(Err(Error::new(
            ErrorType::TypeConversionError("Signedness of compared types does not match"),
            fn_path.span(),
        )));
    }

    let fn_prefix = if left_is_signed { "s" } else { "u" };

    // strength of inequality is preserved when arguments are swapped
    // i.e. a >= b becomes b <= a, a > b becomes b < a
    let (fn_suffix, swap_args) = match fn_path.segments[3].ident.name() {
        "lt" => ("lt", false),
        "le" => ("le", false),
        "gt" => ("lt", true),
        "ge" => ("le", true),
        _ => panic!("Unexpected comparison function"),
    };
    if swap_args {
        args.swap(0, 1);
    }

    fn_path.segments[3]
        .ident
        .set_name(format!("{}{}", fn_prefix, fn_suffix));

    Some(Ok(convert_basic_path(fn_path)))
}

fn convert_shift(
    fn_path: &WPath<WBasicType>,
    args: &mut [WCallArg],
    local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
) -> Option<Result<WPath<WElementaryType>, Error>> {
    // --- Shl ---
    if fn_path.matches_absolute(&["std", "ops", "Shl", "shl"]) {
        let mut fn_path = fn_path.clone();
        fn_path.segments[0].ident.set_name(String::from("mck"));
        fn_path.segments[1].ident.set_name(String::from("forward"));
        fn_path.segments[2].ident.set_name(String::from("HwShift"));
        fn_path.segments[3]
            .ident
            .set_name(String::from("logic_shl"));
        return Some(Ok(convert_basic_path(fn_path)));
    }

    // --- Shr ---
    if fn_path.matches_absolute(&["std", "ops", "Shr", "shr"]) {
        let mut fn_path = fn_path.clone();
        fn_path.segments[0].ident.set_name(String::from("mck"));
        fn_path.segments[1].ident.set_name(String::from("forward"));
        fn_path.segments[2].ident.set_name(String::from("HwShift"));

        // determine signedness from the first argument
        // note that in Rust, type inference depends on whether Shr is an operation or a call
        // but this should not impact our simple bitvector-signed-unsigned types
        if args.len() != 2 {
            return Some(Err(Error::new(
                ErrorType::TypeConversionError("Right shift should have exactly two arguments"),
                fn_path.span(),
            )));
        }

        let Some(is_signed) = signedness(&args[0], local_types) else {
            return Some(Err(Error::new(
                ErrorType::TypeConversionError("Cannot determine right shift signedness"),
                fn_path.span(),
            )));
        };

        let fn_name = if is_signed { "arith_shr" } else { "logic_shr" };
        fn_path.segments[3].ident.set_name(String::from(fn_name));
        return Some(Ok(convert_basic_path(fn_path)));
    }
    None
}

fn convert_ext(
    fn_path: &WPath<WBasicType>,
    args: &[WCallArg],
    local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
) -> Option<Result<WPath<WElementaryType>, Error>> {
    if !fn_path.matches_absolute(&["machine_check", "Ext", "ext"]) {
        return None;
    }

    // need to know type signedness
    if args.len() != 1 {
        return Some(Err(Error::new(
            ErrorType::TypeConversionError("Bit extension should have exactly one argument"),
            fn_path.span(),
        )));
    }
    let mut fn_path = path_start_to_mck_forward(fn_path.clone());

    let Some(is_signed) = signedness(&args[0], local_types) else {
        return Some(Err(Error::new(
            ErrorType::TypeConversionError("Cannot determine bit extension signedness"),
            fn_path.span(),
        )));
    };

    let fn_name = if is_signed { "sext" } else { "uext" };
    fn_path.segments[3].ident.set_name(String::from(fn_name));
    Some(Ok(convert_basic_path(fn_path)))
}

fn convert_into(
    orig_call: WExprHighCall<WBasicType>,
    local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
) -> Option<Result<WExpr<WElementaryType, WExprCall<WElementaryType>>, Error>> {
    let WExprHighCall::Call(WCall { fn_path, args }) = orig_call;
    if !fn_path.matches_absolute(&["std", "convert", "Into", "into"]) {
        return None;
    }
    // make sure the argument is a bitvector-related type
    // we do not need to check generics as these will be handled by Rust
    if args.len() != 1 {
        return Some(Err(Error::new(
            ErrorType::TypeConversionError("Into should have exactly one argument"),
            fn_path.span(),
        )));
    }

    let arg_ident = match &args[0] {
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
        return Some(Err(Error::new(
            ErrorType::TypeConversionError(
                "Unable to assure that argument to Into is a machine-check bitvector",
            ),
            fn_path.span(),
        )));
    };
    // into is no-op for our converted types, so change to a move
    Some(Ok(WExpr::Move(arg_ident)))
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
