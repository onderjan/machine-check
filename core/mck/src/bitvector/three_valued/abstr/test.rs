use crate::forward::*;

use super::*;

// --- ANECDOTAL TESTS ---

#[test]
pub fn support() {
    let cafe = ThreeValuedBitvector::<16>::new(0xCAFE);
    assert_eq!(
        cafe.get_possibly_zero_flags(),
        concr::Bitvector::new(0x3501)
    );
    assert_eq!(cafe.get_possibly_one_flags(), concr::Bitvector::new(0xCAFE));
    assert_eq!(cafe.get_unknown_bits(), concr::Bitvector::new(0));
    assert_eq!(cafe.concrete_value(), Some(concr::Bitvector::new(0xCAFE)));
    assert!(cafe.contains_concr(&concr::Bitvector::new(0xCAFE)));
    assert!(!cafe.contains_concr(&concr::Bitvector::new(0xCAFF)));

    let unknown = ThreeValuedBitvector::<16>::new_unknown();
    assert_eq!(
        unknown.get_possibly_zero_flags(),
        concr::Bitvector::<16>::new(0xFFFF)
    );
    assert_eq!(
        unknown.get_possibly_one_flags(),
        concr::Bitvector::<16>::new(0xFFFF)
    );
    assert_eq!(unknown.get_unknown_bits(), concr::Bitvector::new(0xFFFF));
    assert_eq!(unknown.concrete_value(), None);
    assert!(unknown.contains_concr(&concr::Bitvector::new(0xCAFE)));
    assert!(unknown.contains_concr(&concr::Bitvector::new(0xCAFF)));

    let partially_known = ThreeValuedBitvector::<16>::new_value_known(
        concr::Bitvector::new(0x1337),
        concr::Bitvector::new(0xF0F0),
    );
    assert_eq!(
        partially_known.get_possibly_zero_flags(),
        concr::Bitvector::<16>::new(0xEFCF)
    );
    assert_eq!(
        partially_known.get_possibly_one_flags(),
        concr::Bitvector::<16>::new(0x1F3F)
    );
    assert_eq!(
        partially_known.get_unknown_bits(),
        concr::Bitvector::new(0x0F0F)
    );
    assert_eq!(partially_known.concrete_value(), None);
    assert!(partially_known.contains_concr(&concr::Bitvector::new(0x1337)));
    assert!(partially_known.contains_concr(&concr::Bitvector::new(0x1D30)));
    assert!(!partially_known.contains_concr(&concr::Bitvector::new(0xCAFE)));
    assert!(!partially_known.contains_concr(&concr::Bitvector::new(0xCAFF)));

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
        cafe.concrete_join(concr::Bitvector::new(0x1337)),
        ThreeValuedBitvector::from_zeros_ones(
            concr::Bitvector::new(0xFDC9),
            concr::Bitvector::new(0xDBFF)
        )
    );

    assert_eq!(
        ThreeValuedBitvector::<8>::all_of_length_iter().count(),
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
        concr::Bitvector::new(0xFFEC),
        concr::Bitvector::new(0xF34F),
    );
}

// --- EXHAUSTIVE TESTS ---

macro_rules! uni_op_test {
    ($op:tt) => {
        seq_macro::seq!(L in 0..=8 {

        #[test]
        pub fn $op~L() {
            let abstr_func = |a: ThreeValuedBitvector<L>| a.$op();
            let concr_func = |a: concr::Bitvector<L>| a.$op();
            exec_uni_check(abstr_func, concr_func);
        }
    });
    };
}

macro_rules! ext_op_test {
    ($op:tt) => {
        seq_macro::seq!(L in 0..=6 {
            seq_macro::seq!(X in 0..=6 {
                #[test]
                pub fn $op~L~X() {
                    let abstr_func =
                        |a: ThreeValuedBitvector<L>| -> ThreeValuedBitvector<X> { a.$op() };
                    let concr_func = |a: concr::Bitvector<L>| -> concr::Bitvector<X> { a.$op() };
                    exec_uni_check(abstr_func, concr_func);
                }
            });
        });
    };
}

macro_rules! bi_op_test {
    ($op:tt,$exact:tt) => {

        seq_macro::seq!(L in 0..=6 {

        #[test]
        pub fn $op~L() {
            let abstr_func = |a: ThreeValuedBitvector<L>, b: ThreeValuedBitvector<L>| a.$op(b);
            let concr_func = |a: concr::Bitvector<L>, b: concr::Bitvector<L>| a.$op(b);
            exec_bi_check(abstr_func, concr_func, $exact);
        }
    });
    };
}

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

fn exec_uni_check<const L: u32, const X: u32>(
    abstr_func: fn(ThreeValuedBitvector<L>) -> ThreeValuedBitvector<X>,
    concr_func: fn(concr::Bitvector<L>) -> concr::Bitvector<X>,
) {
    for a in ThreeValuedBitvector::<L>::all_of_length_iter() {
        let abstr_result = abstr_func(a);
        let equiv_result = join_concr_iter(
            concr::Bitvector::<L>::all_of_length_iter()
                .filter(|c| a.contains_concr(c))
                .map(concr_func),
        );
        if abstr_result != equiv_result {
            panic!(
                "Wrong result with parameter {}, expected {}, got {}",
                a, equiv_result, abstr_result
            );
        }
    }
}

fn exec_bi_check<const L: u32, const X: u32>(
    abstr_func: fn(ThreeValuedBitvector<L>, ThreeValuedBitvector<L>) -> ThreeValuedBitvector<X>,
    concr_func: fn(concr::Bitvector<L>, concr::Bitvector<L>) -> concr::Bitvector<X>,
    exact: bool,
) {
    for a in ThreeValuedBitvector::<L>::all_of_length_iter() {
        for b in ThreeValuedBitvector::<L>::all_of_length_iter() {
            let abstr_result = abstr_func(a, b);

            let a_concr_iter =
                concr::Bitvector::<L>::all_of_length_iter().filter(|c| a.contains_concr(c));
            let equiv_result = join_concr_iter(a_concr_iter.flat_map(|a_concr| {
                concr::Bitvector::<L>::all_of_length_iter()
                    .filter(|c| b.contains_concr(c))
                    .map(move |b_concr| concr_func(a_concr, b_concr))
            }));

            if exact {
                if abstr_result != equiv_result {
                    panic!(
                        "Non-exact result with parameters {}, {}, expected {}, got {}",
                        a, b, equiv_result, abstr_result
                    );
                }
            } else if !abstr_result.contains(&equiv_result) {
                panic!(
                    "Unsound result with parameters {}, {}, expected {}, got {}",
                    a, b, equiv_result, abstr_result
                );
            }
            if a.concrete_value().is_some()
                && b.concrete_value().is_some()
                && abstr_result.concrete_value().is_none()
            {
                panic!(
                            "Non-concrete-value result with concrete-value parameters {}, {}, expected {}, got {}",
                            a, b, equiv_result, abstr_result
                        );
            }
        }
    }
}

fn join_concr_iter<const L: u32>(
    mut iter: impl Iterator<Item = concr::Bitvector<L>>,
) -> ThreeValuedBitvector<L> {
    if L == 0 {
        return ThreeValuedBitvector::new_unknown();
    }

    let first_concrete = iter
        .next()
        .expect("Expected at least one concrete bitvector in iterator");

    let mut result = ThreeValuedBitvector::from_concrete(first_concrete);

    for c in iter {
        result = result.concrete_join(c)
    }
    result
}
