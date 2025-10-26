#[macro_use]
mod op;

use crate::{
    abstr::three_valued::RThreeValuedBitvector,
    misc::RBound,
    refin::RefinementDomain,
    traits::misc::{Meta, MetaEq},
};

use super::*;

// === ANECDOTAL TESTS ===

#[test]
pub fn support() {
    let unmarked = RMarkBitvector::new_unmarked(16);
    assert_eq!(
        unmarked.marked_bits(),
        RConcreteBitvector::new(0x0000, RBound::new(16))
    );

    let marked = RMarkBitvector::new_marked_unimportant(16);
    assert_eq!(
        marked.marked_bits(),
        RConcreteBitvector::new(0xFFFF, RBound::new(16))
    );

    let cafe = RMarkBitvector::new_from_flag(RConcreteBitvector::new(0xCAFE, RBound::new(16)));
    assert_eq!(
        cafe.marked_bits(),
        RConcreteBitvector::new(0xCAFE, RBound::new(16))
    );

    let known = RThreeValuedBitvector::new(0xBABE, RBound::new(16));
    assert_eq!(unmarked.limit(&known), unmarked);
    assert_eq!(marked.limit(&known), unmarked);
    assert_eq!(cafe.limit(&known), unmarked);

    let half_known = RThreeValuedBitvector::new_value_known(
        RConcreteBitvector::new(0xBABE, RBound::new(16)),
        RConcreteBitvector::new(0xF000, RBound::new(16)),
    );
    assert_eq!(unmarked.limit(&half_known), unmarked);
    assert_eq!(
        marked.limit(&half_known),
        RMarkBitvector::new_from_flag(RConcreteBitvector::new(0x0FFF, RBound::new(16)))
    );
    assert_eq!(
        cafe.limit(&half_known),
        RMarkBitvector::new_from_flag(RConcreteBitvector::new(0x0AFE, RBound::new(16)))
    );
}

#[test]
pub fn meta() {
    // should represent two three-valued bitvectors "XX0X" and "XX1X"
    let mark = RMarkBitvector::new_from_flag(RConcreteBitvector::new(0x2, RBound::new(4)));

    let mut v = mark.proto_first();
    assert!(v.meta_eq(
        // "XX0X"
        &RThreeValuedBitvector::new_value_known(
            RConcreteBitvector::new(0x0, RBound::new(4)),
            RConcreteBitvector::new(0x2, RBound::new(4))
        )
    ));
    assert!(mark.proto_increment(&mut v));
    assert!(v.meta_eq(
        // "XX1X"
        &RThreeValuedBitvector::new_value_known(
            RConcreteBitvector::new(0x2, RBound::new(4)),
            RConcreteBitvector::new(0x2, RBound::new(4))
        )
    ));
    // returns false due to cycling, but v should contain the first proto again
    assert!(!mark.proto_increment(&mut v));
    assert!(v.meta_eq(
        // "XX0X"
        &RThreeValuedBitvector::new_value_known(
            RConcreteBitvector::new(0x0, RBound::new(4)),
            RConcreteBitvector::new(0x2, RBound::new(4))
        )
    ));
}

#[test]
pub fn refine() {
    let mark_a = RMarkBitvector::new_from_flag(RConcreteBitvector::new(0x2, RBound::new(4)));
    let mut mark_b = RMarkBitvector::new_from_flag(RConcreteBitvector::new(0x4, RBound::new(4)));
    mark_b.apply_join(&mark_a);

    // applies all bits
    assert_eq!(
        mark_b,
        RMarkBitvector::new_from_flag(RConcreteBitvector::new(0x6, RBound::new(4)))
    );

    let mut mark_c = RMarkBitvector::new_from_flag(RConcreteBitvector::new(0x1, RBound::new(4)));
    // applies only the highest bit
    assert!(mark_c.apply_refin(&mark_b));
    assert_eq!(
        mark_c,
        RMarkBitvector::new_from_flag(RConcreteBitvector::new(0x5, RBound::new(4)))
    );

    assert!(!mark_b.apply_refin(&mark_a));

    let mut three_valued = RThreeValuedBitvector::new(0xC, RBound::new(4));
    mark_c.force_decay(&mut three_valued);
    // unmarked fields become unknown
    assert!(
        three_valued.meta_eq(&RThreeValuedBitvector::from_zeros_ones(
            RConcreteBitvector::new(0xB, RBound::new(4)),
            RConcreteBitvector::new(0xE, RBound::new(4))
        ))
    )
}

// === SMALL-LENGTH-EXHAUSTIVE TESTS ===

// --- UNARY TESTS ---

uni_op_test!(Bitwise, bit_not, true);
uni_op_test!(HwArith, arith_neg, false);

// --- BINARY TESTS ---

// arithmetic tests
bi_op_test!(HwArith, add, false);
bi_op_test!(HwArith, sub, false);
bi_op_test!(HwArith, mul, false);

divrem_op_test!(HwArith, sdiv, false);
divrem_op_test!(HwArith, udiv, false);
divrem_op_test!(HwArith, srem, false);
divrem_op_test!(HwArith, urem, false);

// bitwise tests
bi_op_test!(Bitwise, bit_and, false);
bi_op_test!(Bitwise, bit_or, false);
bi_op_test!(Bitwise, bit_xor, false);

// equality and comparison tests
cmp_op_test!(TypedEq, eq, false);
cmp_op_test!(TypedCmp, slt, false);
cmp_op_test!(TypedCmp, sle, false);
cmp_op_test!(TypedCmp, ult, false);
cmp_op_test!(TypedCmp, ule, false);

// shift tests
bi_op_test!(HwShift, logic_shl, false);
bi_op_test!(HwShift, logic_shr, false);
bi_op_test!(HwShift, arith_shr, false);

// --- EXTENSION TESTS ---

// extension tests
ext_op_test!(RExt, uext, false);
ext_op_test!(RExt, sext, false);
