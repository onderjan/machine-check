#[macro_use]
mod op;

use crate::{
    bitvector::three_valued::abstr::ThreeValuedBitvector, refin::Refine, traits::misc::Meta,
};

use super::*;

// === ANECDOTAL TESTS ===

#[test]
pub fn support() {
    let unmarked = MarkBitvector::<16>::new_unmarked();
    assert_eq!(unmarked.0, ConcreteBitvector::new(0x0000));

    let marked = MarkBitvector::<16>::new_marked();
    assert_eq!(marked.0, ConcreteBitvector::new(0xFFFF));

    let cafe = MarkBitvector::<16>::new_from_flag(ConcreteBitvector::new(0xCAFE));
    assert_eq!(cafe.0, ConcreteBitvector::new(0xCAFE));

    let known = ThreeValuedBitvector::new(0xBABE);
    assert_eq!(unmarked.limit(known), unmarked);
    assert_eq!(marked.limit(known), unmarked);
    assert_eq!(cafe.limit(known), unmarked);

    let half_known = ThreeValuedBitvector::new_value_known(
        ConcreteBitvector::new(0xBABE),
        ConcreteBitvector::new(0xF000),
    );
    assert_eq!(unmarked.limit(half_known), unmarked);
    assert_eq!(
        marked.limit(half_known),
        MarkBitvector::new_from_flag(ConcreteBitvector::new(0x0FFF))
    );
    assert_eq!(
        cafe.limit(half_known),
        MarkBitvector::new_from_flag(ConcreteBitvector::new(0x0AFE))
    );
}

#[test]
pub fn meta() {
    // should represent two three-valued bitvectors "XX0X" and "XX1X"
    let mark = MarkBitvector::<4>::new_from_flag(ConcreteBitvector::new(0x2));

    let mut v = mark.proto_first();
    assert_eq!(
        v,
        // "XX0X"
        ThreeValuedBitvector::new_value_known(
            ConcreteBitvector::new(0x0),
            ConcreteBitvector::new(0x2)
        )
    );
    assert!(mark.proto_increment(&mut v));
    assert_eq!(
        v,
        // "XX1X"
        ThreeValuedBitvector::new_value_known(
            ConcreteBitvector::new(0x2),
            ConcreteBitvector::new(0x2)
        )
    );
    // returns false due to cycling, but v should contain the first proto again
    assert!(!mark.proto_increment(&mut v));
    assert_eq!(
        v,
        // "XX0X"
        ThreeValuedBitvector::new_value_known(
            ConcreteBitvector::new(0x0),
            ConcreteBitvector::new(0x2)
        )
    );
}

#[test]
pub fn refine() {
    let mark_a = MarkBitvector::<4>::new_from_flag(ConcreteBitvector::new(0x2));
    let mut mark_b = MarkBitvector::<4>::new_from_flag(ConcreteBitvector::new(0x4));
    mark_b.apply_join(&mark_a);

    // applies all bits
    assert_eq!(
        mark_b,
        MarkBitvector::new_from_flag(ConcreteBitvector::new(0x6))
    );

    let mut mark_c = MarkBitvector::<4>::new_from_flag(ConcreteBitvector::new(0x1));
    // applies only the highest bit
    assert!(mark_c.apply_refin(&mark_b));
    assert_eq!(
        mark_c,
        MarkBitvector::new_from_flag(ConcreteBitvector::new(0x5))
    );

    assert!(!mark_b.apply_refin(&mark_a));

    let mut three_valued = ThreeValuedBitvector::new(0xC);
    mark_c.force_decay(&mut three_valued);
    // unmarked fields become unknown
    assert_eq!(
        three_valued,
        ThreeValuedBitvector::from_zeros_ones(
            ConcreteBitvector::new(0xB),
            ConcreteBitvector::new(0xE)
        )
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
bi_op_test!(HwArith, sdiv, false);
bi_op_test!(HwArith, udiv, false);
bi_op_test!(HwArith, srem, false);
bi_op_test!(HwArith, urem, false);

// bitwise tests
bi_op_test!(Bitwise, bit_and, false);
bi_op_test!(Bitwise, bit_or, false);
bi_op_test!(Bitwise, bit_xor, false);

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
