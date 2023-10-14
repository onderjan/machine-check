#[macro_use]
mod op;

use super::*;

// --- UNARY TESTS ---

uni_op_test!(Bitwise, not, true);
uni_op_test!(HwArith, neg, false);

// --- BINARY TESTS ---

// arithmetic tests
bi_op_test!(HwArith, add, false);
bi_op_test!(HwArith, sub, false);
bi_op_test!(HwArith, mul, false);
bi_op_test!(HwArith, sdiv, false);
bi_op_test!(HwArith, udiv, false);
bi_op_test!(HwArith, srem, false);
bi_op_test!(HwArith, urem, false);

// bitwise tests
bi_op_test!(Bitwise, bitand, false);
bi_op_test!(Bitwise, bitor, false);
bi_op_test!(Bitwise, bitxor, false);

// equality and comparison tests
bi_op_test!(TypedEq, typed_eq, false);
bi_op_test!(TypedCmp, typed_slt, false);
bi_op_test!(TypedCmp, typed_slte, false);
bi_op_test!(TypedCmp, typed_ult, false);
bi_op_test!(TypedCmp, typed_ulte, false);

// shift tests
bi_op_test!(HwShift, logic_shl, false);
bi_op_test!(HwShift, logic_shr, false);
bi_op_test!(HwShift, arith_shr, false);

// --- EXTENSION TESTS ---

// extension tests
ext_op_test!(Ext, uext, false);
ext_op_test!(Ext, sext, false);
