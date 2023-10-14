use crate::bitvector::{refin, three_valued::abstr::ThreeValuedBitvector, util};

use super::*;
fn exact_uni_mark<const L: u32, const X: u32>(
    a_abstr: ThreeValuedBitvector<L>,
    a_mark: refin::Bitvector<X>,
    concr_func: fn(concr::Bitvector<L>) -> concr::Bitvector<X>,
) -> MarkBitvector<L> {
    // the result marks exactly those bits of input which, if changed in operation input,
    // can change bits masked by mark_a in the operation result
    let mark_mask = a_mark.0.as_unsigned();
    // determine for each input bit separately
    let mut result = 0;
    for i in 0..L {
        for a in 0..(1 << L) {
            if a & (1 << i) != 0 {
                continue;
            }
            let with_zero = concr::Bitvector::new(a);
            let with_one = concr::Bitvector::new(a | (1 << i));
            if !a_abstr.contains_concr(&with_zero) || !a_abstr.contains_concr(&with_one) {
                continue;
            }
            if concr_func(with_zero).as_unsigned() & mark_mask
                != concr_func(with_one).as_unsigned() & mark_mask
            {
                result |= 1 << i;
            }
        }
    }
    MarkBitvector(concr::Bitvector::new(result))
}

fn exec_uni_check<const L: u32, const X: u32>(
    mark_func: fn(ThreeValuedBitvector<L>, MarkBitvector<X>) -> MarkBitvector<L>,
    concr_func: fn(concr::Bitvector<L>) -> concr::Bitvector<X>,
    want_exact: bool,
) {
    // a mark bit is necessary if changing the input bit can impact the output
    // test this for all concretizations of the input

    let mask = util::compute_u64_mask(L);
    for a_mark in 0..(1 << X) {
        let a_mark = MarkBitvector(concr::Bitvector::new(a_mark));

        for a_zeros in 0..(1 << L) {
            for a_ones in 0..(1 << L) {
                if (a_zeros | a_ones) & mask != mask {
                    continue;
                }
                let a_zeros = concr::Bitvector::new(a_zeros);
                let a_ones = concr::Bitvector::new(a_ones);
                let a_abstr = ThreeValuedBitvector::<L>::from_zeros_ones(a_zeros, a_ones);
                let exact_earlier_mark = exact_uni_mark(a_abstr, a_mark, concr_func);
                let our_earlier_mark = mark_func(a_abstr, a_mark);

                if want_exact {
                    // test for exactness
                    if exact_earlier_mark != our_earlier_mark {
                        panic!(
                            "Non-exact for earlier mark with input {} and later mark {}, expected {}, got {}",
                            a_abstr, a_mark.0, exact_earlier_mark.0, our_earlier_mark.0
                        );
                    }
                } else {
                    // test whether our earlier mark is at least as marked as the exact one
                    // if not, the marking will be incomplete
                    let exact = exact_earlier_mark.0.as_unsigned();
                    let our = our_earlier_mark.0.as_unsigned();
                    if our & exact != exact {
                        panic!(
                            "Incomplete for earlier mark with input {} and later mark {}, expected {}, got {}",
                            a_abstr, a_mark.0, exact_earlier_mark.0, our_earlier_mark.0
                        );
                    }
                    // TODO: test for spurious marking when no later mark was passed
                }
            }
        }
    }
    // test unprovoked marking

    for a_zeros in 0..(1 << L) {
        for a_ones in 0..(1 << L) {
            if (a_zeros | a_ones) & mask != mask {
                continue;
            }
            let a_zeros = concr::Bitvector::new(a_zeros);
            let a_ones = concr::Bitvector::new(a_ones);
            let a_abstr = ThreeValuedBitvector::<L>::from_zeros_ones(a_zeros, a_ones);
            let a_mark = MarkBitvector::new_unmarked();
            let our_earlier_mark = mark_func(a_abstr, a_mark);
            if our_earlier_mark != MarkBitvector::new_unmarked() {
                panic!(
                    "Unprovoked marking with inputs {}, got mark {}",
                    a_abstr, our_earlier_mark.0
                );
            }
        }
    }
}

macro_rules! uni_op_test {
    ($ty:tt, $op:tt, $exact:tt) => {

        seq_macro::seq!(L in 0..=6 {

        #[test]
        pub fn $op~L() {
            let mark_func = |a: ThreeValuedBitvector<L>,
                                a_mark: MarkBitvector<L>|
                -> MarkBitvector<L> { crate::backward::$ty::$op((a,), a_mark).0 };
            let concr_func = crate::forward::$ty::$op;
            exec_uni_check(mark_func, concr_func, $exact);
        }
    });
    };
}

macro_rules! ext_op_test {
    ($ty:tt, $op:tt, $exact:tt) => {

        seq_macro::seq!(L in 0..=6 {
            seq_macro::seq!(X in 0..=6 {

            #[test]
            pub fn $op~L~X() {
                let mark_func = |a: ThreeValuedBitvector<L>,
                                a_mark: MarkBitvector<X>|
                -> MarkBitvector<L> { crate::backward::$ty::$op((a,), a_mark).0 };
                let concr_func = crate::forward::$ty::$op;
                exec_uni_check(mark_func, concr_func, $exact);
            }
            });
        });
    };
}

fn exact_bi_mark<const L: u32, const X: u32>(
    abstr: (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
    mark: MarkBitvector<X>,
    concr_func: fn(concr::Bitvector<L>, concr::Bitvector<L>) -> concr::Bitvector<X>,
) -> (MarkBitvector<L>, MarkBitvector<L>) {
    let a_abstr = abstr.0;
    let b_abstr = abstr.1;
    // the result marks exactly those bits of input which, if changed in operation input,
    // can change bits masked by mark_a in the operation result
    let mark_mask = mark.0.as_unsigned();
    // determine for each input bit separately
    let mut a_result = 0;
    let mut b_result = 0;
    for i in 0..L {
        for our in 0..(1 << L) {
            if our & (1 << i) != 0 {
                continue;
            }
            let with_zero = concr::Bitvector::new(our);
            let with_one = concr::Bitvector::new(our | (1 << i));
            if !a_abstr.contains_concr(&with_zero) || !a_abstr.contains_concr(&with_one) {
                continue;
            }
            for other in 0..(1 << L) {
                if !b_abstr.contains_concr(&concr::Bitvector::new(other)) {
                    continue;
                }
                let other = concr::Bitvector::new(other);
                if concr_func(with_zero, other).as_unsigned() & mark_mask
                    != concr_func(with_one, other).as_unsigned() & mark_mask
                {
                    a_result |= 1 << i;
                }
            }
        }
    }
    for i in 0..L {
        for our in 0..(1 << L) {
            if our & (1 << i) != 0 {
                continue;
            }
            let with_zero = concr::Bitvector::new(our);
            let with_one = concr::Bitvector::new(our | (1 << i));
            if !b_abstr.contains_concr(&with_zero) || !b_abstr.contains_concr(&with_one) {
                continue;
            }
            for other in 0..(1 << L) {
                let other = concr::Bitvector::new(other);
                if !a_abstr.contains_concr(&other) {
                    continue;
                }
                if concr_func(other, with_zero).as_unsigned() & mark_mask
                    != concr_func(other, with_one).as_unsigned() & mark_mask
                {
                    b_result |= 1 << i;
                }
            }
        }
    }
    (
        MarkBitvector(concr::Bitvector::new(a_result)),
        MarkBitvector(concr::Bitvector::new(b_result)),
    )
}

fn exec_bi_check<const L: u32, const X: u32>(
    mark_func: fn(
        (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
        MarkBitvector<X>,
    ) -> (MarkBitvector<L>, MarkBitvector<L>),
    concr_func: fn(concr::Bitvector<L>, concr::Bitvector<L>) -> concr::Bitvector<X>,
    want_exact: bool,
) {
    // a mark bit is necessary if changing the input bit can impact the output
    // test this for all concretizations of the input

    let mask = util::compute_u64_mask(L);
    for a_mark in 0..(1 << X) {
        let a_mark = MarkBitvector(concr::Bitvector::new(a_mark));

        for a_zeros in 0..(1 << L) {
            for a_ones in 0..(1 << L) {
                if (a_zeros | a_ones) & mask != mask {
                    continue;
                }
                let a_zeros = concr::Bitvector::new(a_zeros);
                let a_ones = concr::Bitvector::new(a_ones);
                let a_abstr = ThreeValuedBitvector::<L>::from_zeros_ones(a_zeros, a_ones);
                for b_zeros in 0..(1 << L) {
                    for b_ones in 0..(1 << L) {
                        if (b_zeros | b_ones) & mask != mask {
                            continue;
                        }
                        let b_zeros = concr::Bitvector::new(b_zeros);
                        let b_ones = concr::Bitvector::new(b_ones);
                        let b_abstr = ThreeValuedBitvector::<L>::from_zeros_ones(b_zeros, b_ones);
                        let exact_earlier_mark =
                            exact_bi_mark((a_abstr, b_abstr), a_mark, concr_func);
                        let our_earlier_mark = mark_func((a_abstr, b_abstr), a_mark);

                        if want_exact {
                            // test for exactness
                            if exact_earlier_mark != our_earlier_mark {
                                panic!(
                                    "Non-exact for earlier mark with inputs ({}, {}) and later mark {}, expected ({}, {}), got ({}, {})",
                                    a_abstr, b_abstr, a_mark.0, exact_earlier_mark.0.0, exact_earlier_mark.1.0, our_earlier_mark.0.0, our_earlier_mark.1.0
                                );
                            }
                        } else {
                            // test whether our earlier mark is at least as marked as the exact one
                            // if not, the marking will be incomplete
                            let a_exact = exact_earlier_mark.0 .0.as_unsigned();
                            let b_exact = exact_earlier_mark.1 .0.as_unsigned();
                            let a_our = our_earlier_mark.0 .0.as_unsigned();
                            let b_our = our_earlier_mark.1 .0.as_unsigned();
                            if a_our & a_exact != a_exact || b_our & b_exact != b_exact {
                                panic!(
                                    "Incomplete for earlier mark with inputs ({}, {}) and later mark {}, expected ({}, {}), got ({}, {})",
                                    a_abstr, b_abstr, a_mark.0, exact_earlier_mark.0.0, exact_earlier_mark.1.0, our_earlier_mark.0.0, our_earlier_mark.1.0
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    // TODO: other way for marks

    // test unprovoked marking

    for a_zeros in 0..(1 << L) {
        for a_ones in 0..(1 << L) {
            if (a_zeros | a_ones) & mask != mask {
                continue;
            }
            let a_zeros = concr::Bitvector::new(a_zeros);
            let a_ones = concr::Bitvector::new(a_ones);
            let a_abstr = ThreeValuedBitvector::<L>::from_zeros_ones(a_zeros, a_ones);
            for b_zeros in 0..(1 << L) {
                for b_ones in 0..(1 << L) {
                    if (b_zeros | b_ones) & mask != mask {
                        continue;
                    }
                    let b_zeros = concr::Bitvector::new(b_zeros);
                    let b_ones = concr::Bitvector::new(b_ones);
                    let b_abstr = ThreeValuedBitvector::<L>::from_zeros_ones(b_zeros, b_ones);
                    let a_mark = MarkBitvector::new_unmarked();
                    let our_earlier_mark = mark_func((a_abstr, b_abstr), a_mark);
                    if our_earlier_mark.0 != MarkBitvector::new_unmarked()
                        || our_earlier_mark.1 != MarkBitvector::new_unmarked()
                    {
                        panic!(
                            "Unprovoked marking with inputs ({}, {}), got mark ({}, {})",
                            a_abstr, b_abstr, our_earlier_mark.0 .0, our_earlier_mark.1 .0
                        );
                    }
                }
            }
        }
    }
}

macro_rules! bi_op_test {
    ($ty:tt, $op:tt, $exact:tt) => {

        seq_macro::seq!(L in 0..=3 {

        #[test]
        pub fn $op~L() {
            let mark_func = |inputs: (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
                                mark| {
                crate::backward::$ty::$op(inputs, mark)
            };
            let concr_func = crate::forward::$ty::$op;
            exec_bi_check(mark_func, concr_func, $exact);
        }
    });
    };
}

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
