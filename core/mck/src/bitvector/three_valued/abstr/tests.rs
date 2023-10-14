#[macro_use]
mod op;

use super::*;
use crate::forward::*;

// === ANECDOTAL TESTS ===

#[test]
pub fn support() {
    let cafe = ThreeValuedBitvector::<16>::new(0xCAFE);
    assert_eq!(
        cafe.get_possibly_zero_flags(),
        ConcreteBitvector::new(0x3501)
    );
    assert_eq!(
        cafe.get_possibly_one_flags(),
        ConcreteBitvector::new(0xCAFE)
    );
    assert_eq!(cafe.get_unknown_bits(), ConcreteBitvector::new(0));
    assert_eq!(cafe.concrete_value(), Some(ConcreteBitvector::new(0xCAFE)));
    assert!(cafe.contains_concr(&ConcreteBitvector::new(0xCAFE)));
    assert!(!cafe.contains_concr(&ConcreteBitvector::new(0xCAFF)));

    let unknown = ThreeValuedBitvector::<16>::new_unknown();
    assert_eq!(
        unknown.get_possibly_zero_flags(),
        ConcreteBitvector::<16>::new(0xFFFF)
    );
    assert_eq!(
        unknown.get_possibly_one_flags(),
        ConcreteBitvector::<16>::new(0xFFFF)
    );
    assert_eq!(unknown.get_unknown_bits(), ConcreteBitvector::new(0xFFFF));
    assert_eq!(unknown.concrete_value(), None);
    assert!(unknown.contains_concr(&ConcreteBitvector::new(0xCAFE)));
    assert!(unknown.contains_concr(&ConcreteBitvector::new(0xCAFF)));

    let partially_known = ThreeValuedBitvector::<16>::new_value_known(
        ConcreteBitvector::new(0x1337),
        ConcreteBitvector::new(0xF0F0),
    );
    assert_eq!(
        partially_known.get_possibly_zero_flags(),
        ConcreteBitvector::<16>::new(0xEFCF)
    );
    assert_eq!(
        partially_known.get_possibly_one_flags(),
        ConcreteBitvector::<16>::new(0x1F3F)
    );
    assert_eq!(
        partially_known.get_unknown_bits(),
        ConcreteBitvector::new(0x0F0F)
    );
    assert_eq!(partially_known.concrete_value(), None);
    assert!(partially_known.contains_concr(&ConcreteBitvector::new(0x1337)));
    assert!(partially_known.contains_concr(&ConcreteBitvector::new(0x1D30)));
    assert!(!partially_known.contains_concr(&ConcreteBitvector::new(0xCAFE)));
    assert!(!partially_known.contains_concr(&ConcreteBitvector::new(0xCAFF)));

    assert!(cafe.contains(&cafe));
    assert!(!cafe.contains(&partially_known));
    assert!(!cafe.contains(&unknown));

    assert!(!partially_known.contains(&cafe));
    assert!(partially_known.contains(&partially_known));
    assert!(!partially_known.contains(&unknown));

    assert!(unknown.contains(&cafe));
    assert!(unknown.contains(&partially_known));
    assert!(unknown.contains(&unknown));

    assert_eq!(
        cafe.concrete_join(ConcreteBitvector::new(0x1337)),
        ThreeValuedBitvector::from_zeros_ones(
            ConcreteBitvector::new(0xFDC9),
            ConcreteBitvector::new(0xDBFF)
        )
    );

    assert_eq!(
        ThreeValuedBitvector::<8>::all_with_length_iter().count(),
        3usize.pow(8)
    );
}

#[test]
#[should_panic]
pub fn bitvec_too_large() {
    let _ = ThreeValuedBitvector::<70>::new(0x0924);
}

#[test]
#[should_panic]
pub fn invalid_new() {
    let _ = ThreeValuedBitvector::<3>::new(0x0924);
}

#[test]
#[should_panic]
pub fn invalid_zeros_ones() {
    let _ = ThreeValuedBitvector::<8>::from_zeros_ones(
        ConcreteBitvector::new(0xFFEC),
        ConcreteBitvector::new(0xF34F),
    );
}

// === SMALL-LENGTH-EXHAUSTIVE TESTS ===

// --- UNARY TESTS ---

// not and neg
uni_op_test!(not);

uni_op_test!(neg);

// --- BINARY TESTS ---

// arithmetic tests
bi_op_test!(add, true);
bi_op_test!(sub, true);
bi_op_test!(mul, false);
bi_op_test!(udiv, false);
bi_op_test!(sdiv, false);
bi_op_test!(urem, false);
bi_op_test!(srem, false);

// bitwise tests
bi_op_test!(bitand, true);
bi_op_test!(bitor, true);
bi_op_test!(bitxor, true);

// equality and comparison tests
bi_op_test!(typed_eq, true);
bi_op_test!(typed_slt, true);
bi_op_test!(typed_slte, true);
bi_op_test!(typed_ult, true);
bi_op_test!(typed_ulte, true);

// shift tests
bi_op_test!(logic_shl, true);
bi_op_test!(logic_shr, true);
bi_op_test!(arith_shr, true);

// --- EXTENSION TESTS ---

// extension tests
ext_op_test!(uext);
ext_op_test!(sext);
