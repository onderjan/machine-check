#[macro_use]
mod op;

use super::*;
use crate::{
    abstr::BitvectorDomain,
    concr::CConcreteBitvector,
    forward::*,
    misc::{Join, MetaEq},
};

// === ANECDOTAL TESTS ===

#[test]
pub fn support() {
    let cafe = CThreeValuedBitvector::<16>::new(0xCAFE, CBound);
    assert_eq!(
        cafe.get_possibly_zero_flags(),
        CConcreteBitvector::new(0x3501, CBound)
    );
    assert_eq!(
        cafe.get_possibly_one_flags(),
        CConcreteBitvector::new(0xCAFE, CBound)
    );
    assert_eq!(cafe.get_unknown_bits(), CConcreteBitvector::new(0, CBound));
    assert_eq!(
        cafe.concrete_value(),
        Some(CConcreteBitvector::new(0xCAFE, CBound))
    );
    assert!(cafe.contains_concrete(&CConcreteBitvector::new(0xCAFE, CBound)));
    assert!(!cafe.contains_concrete(&CConcreteBitvector::new(0xCAFF, CBound)));

    let unknown = CThreeValuedBitvector::<16>::new_unknown(CBound);
    assert_eq!(
        unknown.get_possibly_zero_flags(),
        CConcreteBitvector::<16>::new(0xFFFF, CBound)
    );
    assert_eq!(
        unknown.get_possibly_one_flags(),
        CConcreteBitvector::<16>::new(0xFFFF, CBound)
    );
    assert_eq!(
        unknown.get_unknown_bits(),
        CConcreteBitvector::new(0xFFFF, CBound)
    );
    assert_eq!(unknown.concrete_value(), None);
    assert!(unknown.contains_concrete(&CConcreteBitvector::new(0xCAFE, CBound)));
    assert!(unknown.contains_concrete(&CConcreteBitvector::new(0xCAFF, CBound)));

    let partially_known = CThreeValuedBitvector::<16>::new_value_known(
        CConcreteBitvector::new(0x1337, CBound),
        CConcreteBitvector::new(0xF0F0, CBound),
    );
    assert_eq!(
        partially_known.get_possibly_zero_flags(),
        CConcreteBitvector::<16>::new(0xEFCF, CBound)
    );
    assert_eq!(
        partially_known.get_possibly_one_flags(),
        CConcreteBitvector::<16>::new(0x1F3F, CBound)
    );
    assert_eq!(
        partially_known.get_unknown_bits(),
        CConcreteBitvector::new(0x0F0F, CBound)
    );
    assert_eq!(partially_known.concrete_value(), None);
    assert!(partially_known.contains_concrete(&CConcreteBitvector::new(0x1337, CBound)));
    assert!(partially_known.contains_concrete(&CConcreteBitvector::new(0x1D30, CBound)));
    assert!(!partially_known.contains_concrete(&CConcreteBitvector::new(0xCAFE, CBound)));
    assert!(!partially_known.contains_concrete(&CConcreteBitvector::new(0xCAFF, CBound)));

    assert!(cafe.contains(&cafe));
    assert!(!cafe.contains(&partially_known));
    assert!(!cafe.contains(&unknown));

    assert!(!partially_known.contains(&cafe));
    assert!(partially_known.contains(&partially_known));
    assert!(!partially_known.contains(&unknown));

    assert!(unknown.contains(&cafe));
    assert!(unknown.contains(&partially_known));
    assert!(unknown.contains(&unknown));

    assert!(cafe
        .join(&ThreeValuedBitvector::from_concrete_value(
            CConcreteBitvector::new(0x1337, CBound)
        ))
        .meta_eq(&CThreeValuedBitvector::from_zeros_ones(
            CConcreteBitvector::new(0xFDC9, CBound),
            CConcreteBitvector::new(0xDBFF, CBound)
        )));

    assert_eq!(
        CThreeValuedBitvector::<8>::all_with_bound_iter(CBound).count(),
        3usize.pow(8)
    );
}

#[test]
#[should_panic]
pub fn bitvec_too_large() {
    let _ = CThreeValuedBitvector::<70>::new(0x0924, CBound);
}

#[test]
#[should_panic]
pub fn invalid_new() {
    let _ = CThreeValuedBitvector::<3>::new(0x0924, CBound);
}

#[test]
#[should_panic]
pub fn invalid_zeros_ones() {
    let _ = CThreeValuedBitvector::<8>::from_zeros_ones(
        CConcreteBitvector::new(0xFFEC, CBound),
        CConcreteBitvector::new(0xF34F, CBound),
    );
}

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
cmp_op_test!(eq, true);
cmp_op_test!(slt, true);
cmp_op_test!(sle, true);
cmp_op_test!(ult, true);
cmp_op_test!(ule, true);

// shift tests
bi_op_test!(logic_shl, true);
bi_op_test!(logic_shr, true);
bi_op_test!(arith_shr, true);

// --- EXTENSION TESTS ---

// extension tests
ext_op_test!(uext);
ext_op_test!(sext);
