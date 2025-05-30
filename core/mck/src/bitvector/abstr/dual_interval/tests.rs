#[macro_use]
mod op;

use crate::bitvector::abstr::dual_interval::DualInterval;
use crate::bitvector::concr::ConcreteBitvector;
use crate::traits::forward::*;

// === SMALL-LENGTH-EXHAUSTIVE TESTS ===

// --- UNARY TESTS ---

// not and neg
uni_op_test!(bit_not);
uni_op_test!(arith_neg);

// --- BINARY TESTS ---

// arithmetic tests
bi_op_test!(add, true);
bi_op_test!(sub, true);
bi_op_test!(mul, false);
divrem_op_test!(udiv, false);
divrem_op_test!(sdiv, false);
divrem_op_test!(urem, false);
divrem_op_test!(srem, false);

// bitwise tests
bi_op_test!(bit_and, true);
bi_op_test!(bit_or, true);
bi_op_test!(bit_xor, true);

// equality and comparison tests
comparison_op_test!(eq, true);
comparison_op_test!(slt, true);
comparison_op_test!(sle, true);
comparison_op_test!(ult, true);
comparison_op_test!(ule, true);

// shift tests
bi_op_test!(logic_shl, false);
bi_op_test!(logic_shr, false);
bi_op_test!(arith_shr, true);

// --- EXTENSION TESTS ---

// extension tests
ext_op_test!(uext, false);
ext_op_test!(sext, false);
