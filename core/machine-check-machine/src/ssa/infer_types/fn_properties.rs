pub static BIT_RESULT_TRAIT_FNS: [(&str, &str); 5] = [
    ("TypedEq", "typed_eq"),
    ("TypedCmp", "typed_ult"),
    ("TypedCmp", "typed_slt"),
    ("TypedCmp", "typed_ulte"),
    ("TypedCmp", "typed_slte"),
];

pub static TYPE_RETAINING_TRAIT_FNS: [(&str, &str); 15] = [
    ("Bitwise", "bit_not"),
    ("Bitwise", "bit_and"),
    ("Bitwise", "bit_or"),
    ("Bitwise", "bit_xor"),
    ("HwArith", "arith_neg"),
    ("HwArith", "add"),
    ("HwArith", "sub"),
    ("HwArith", "mul"),
    ("HwArith", "udiv"),
    ("HwArith", "sdiv"),
    ("HwArith", "urem"),
    ("HwArith", "srem"),
    ("HwShift", "logic_shl"),
    ("HwShift", "logic_shr"),
    ("HwShift", "arith_shr"),
];

pub static GENERICS_CHANGING_TRAIT_FNS: [(&str, &str); 2] = [("Ext", "uext"), ("Ext", "sext")];
