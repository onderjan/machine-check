pub static STD_CMP_FNS: [(&str, &str); 6] = [
    ("PartialEq", "eq"),
    ("PartialEq", "ne"),
    ("PartialOrd", "lt"),
    ("PartialOrd", "le"),
    ("PartialOrd", "gt"),
    ("PartialOrd", "ge"),
];

pub static STD_OPS_FNS: [(&str, &str); 12] = [
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
