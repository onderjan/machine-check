pub static BIT_RESULT_TRAIT_FNS: [(&str, &str); 5] = [
    ("TypedEq", "typed_eq"),
    ("TypedCmp", "typed_ult"),
    ("TypedCmp", "typed_slt"),
    ("TypedCmp", "typed_ulte"),
    ("TypedCmp", "typed_slte"),
];

pub static TYPE_RETAINING_STD_OPS: [(&str, &str); 12] = [
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

pub static GENERICS_CHANGING_TRAIT_FNS: [(&str, &str); 2] = [("Ext", "uext"), ("Ext", "sext")];
