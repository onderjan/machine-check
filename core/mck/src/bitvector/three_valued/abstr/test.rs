use super::*;

fn join_concr_uni<const L: u32, const X: u32>(
    abstr_a: ThreeValuedBitvector<L>,
    concr_func: fn(concr::Bitvector<L>) -> concr::Bitvector<X>,
) -> ThreeValuedBitvector<X> {
    let x_mask = util::compute_mask(X);
    let mut zeros = Wrapping(0);
    let mut ones = Wrapping(0);
    for a in 0..(1 << L) {
        if !abstr_a.can_contain(Wrapping(a)) {
            continue;
        }
        let a = concr::Bitvector::<L>::new(a);
        let concr_result = concr_func(a);
        zeros |= !concr_result.as_unsigned() & x_mask;
        ones |= concr_result.as_unsigned();
    }
    ThreeValuedBitvector::a_new(zeros, ones)
}

fn exec_uni_check<const L: u32, const X: u32>(
    abstr_func: fn(ThreeValuedBitvector<L>) -> ThreeValuedBitvector<X>,
    concr_func: fn(concr::Bitvector<L>) -> concr::Bitvector<X>,
) {
    let mask = util::compute_mask(L);
    for a_zeros in 0..(1 << L) {
        let a_zeros = Wrapping(a_zeros);
        for a_ones in 0..(1 << L) {
            let a_ones = Wrapping(a_ones);
            if (a_zeros | a_ones) & mask != mask {
                continue;
            }
            let a = ThreeValuedBitvector::<L>::a_new(a_zeros, a_ones);

            let abstr_result = abstr_func(a);
            let equiv_result = join_concr_uni(a, concr_func);
            if abstr_result != equiv_result {
                panic!(
                    "Wrong result with parameter {}, expected {}, got {}",
                    a, equiv_result, abstr_result
                );
            }
        }
    }
}

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

fn join_concr_bi<const L: u32, const X: u32>(
    abstr_a: ThreeValuedBitvector<L>,
    abstr_b: ThreeValuedBitvector<L>,
    concr_func: fn(concr::Bitvector<L>, concr::Bitvector<L>) -> concr::Bitvector<X>,
) -> ThreeValuedBitvector<X> {
    let x_mask = util::compute_mask(X);
    let mut zeros = Wrapping(0);
    let mut ones = Wrapping(0);
    for a in 0..(1 << L) {
        if !abstr_a.can_contain(Wrapping(a)) {
            continue;
        }
        let a = concr::Bitvector::<L>::new(a);
        for b in 0..(1 << L) {
            if !abstr_b.can_contain(Wrapping(b)) {
                continue;
            }
            let b = concr::Bitvector::<L>::new(b);

            let concr_result = concr_func(a, b);
            zeros |= !concr_result.as_unsigned() & x_mask;
            ones |= concr_result.as_unsigned();
        }
    }
    ThreeValuedBitvector::a_new(zeros, ones)
}

fn exec_bi_check<const L: u32, const X: u32>(
    abstr_func: fn(ThreeValuedBitvector<L>, ThreeValuedBitvector<L>) -> ThreeValuedBitvector<X>,
    concr_func: fn(concr::Bitvector<L>, concr::Bitvector<L>) -> concr::Bitvector<X>,
    exact: bool,
) {
    let mask = util::compute_mask(L);
    for a_zeros in 0..(1 << L) {
        let a_zeros = Wrapping(a_zeros);
        for a_ones in 0..(1 << L) {
            let a_ones = Wrapping(a_ones);
            if (a_zeros | a_ones) & mask != mask {
                continue;
            }
            let a = ThreeValuedBitvector::<L>::a_new(a_zeros, a_ones);

            for b_zeros in 0..(1 << L) {
                let b_zeros = Wrapping(b_zeros);
                for b_ones in 0..(1 << L) {
                    let b_ones = Wrapping(b_ones);
                    if (b_zeros | b_ones) & mask != mask {
                        continue;
                    }
                    let b = ThreeValuedBitvector::<L>::a_new(b_zeros, b_ones);

                    let abstr_result = abstr_func(a, b);
                    let equiv_result = join_concr_bi(a, b, concr_func);
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
    }
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
bi_op_test!(sdiv, false);
bi_op_test!(udiv, false);
bi_op_test!(smod, false);
bi_op_test!(seuc, false);
bi_op_test!(urem, false);

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

// misc tests

#[test]
fn shift_test() {
    let a = ThreeValuedBitvector::<64>::a_new(Wrapping(u64::MAX), Wrapping(u64::MAX));
    let b = ThreeValuedBitvector::<64>::a_new(Wrapping(u64::MAX), Wrapping(u64::MAX));
    a.shift(
        b,
        |x, y| {
            assert!(y < u64::BITS as usize);
            x
        },
        |x, y| {
            assert!(y < u64::BITS as usize);
            x
        },
        b,
    );
}
