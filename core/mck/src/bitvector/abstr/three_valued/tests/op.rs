use crate::{
    abstr::{Abstr, BitvectorDomain, Boolean, PanicBitvector, PanicResult},
    bitvector::abstr::three_valued::ThreeValuedBitvector,
    concr::{self, ConcreteBitvector, Test},
    traits::misc::MetaEq,
};

macro_rules! uni_op_test {
    ($op:tt) => {
        seq_macro::seq!(L in 0..=8 {

        #[test]
        pub fn $op~L() {
            let abstr_func = |a: ThreeValuedBitvector<L>| a.$op();
            let concr_func = |a: ConcreteBitvector<L>| a.$op();
            $crate::bitvector::abstr::three_valued::tests::op::exec_uni_check(abstr_func, concr_func);
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
                    let concr_func = |a: ConcreteBitvector<L>| -> ConcreteBitvector<X> { a.$op() };
                    $crate::bitvector::abstr::three_valued::tests::op::exec_uni_check(abstr_func, concr_func);
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
            let abstr_func = |a: ThreeValuedBitvector<L>, b: ThreeValuedBitvector<L>| a.$op(b).into();
            let concr_func = |a: ConcreteBitvector<L>, b: ConcreteBitvector<L>| a.$op(b).into();
            $crate::bitvector::abstr::three_valued::tests::op::exec_bi_check(abstr_func, concr_func, $exact);
        }
    });
    };
}

macro_rules! divrem_op_test {
    ($op:tt,$exact:tt) => {

        seq_macro::seq!(L in 0..=6 {

        #[test]
        pub fn $op~L() {
            let abstr_func = |a: ThreeValuedBitvector<L>, b: ThreeValuedBitvector<L>| a.$op(b).into();
            let concr_func = |a: ConcreteBitvector<L>, b: ConcreteBitvector<L>| a.$op(b).into();
            $crate::bitvector::abstr::three_valued::tests::op::exec_divrem_check(abstr_func, concr_func);
        }
    });
    };
}

pub(super) fn exec_uni_check<const L: u32, const X: u32>(
    abstr_func: fn(ThreeValuedBitvector<L>) -> ThreeValuedBitvector<X>,
    concr_func: fn(ConcreteBitvector<L>) -> ConcreteBitvector<X>,
) {
    for a in ThreeValuedBitvector::<L>::all_with_length_iter() {
        let abstr_result = abstr_func(a);
        let equiv_result = join_concr_iter(
            ConcreteBitvector::<L>::all_with_length_iter()
                .filter(|c| a.contains_concr(c))
                .map(concr_func),
        );
        if !abstr_result.meta_eq(&equiv_result) {
            panic!(
                "Wrong result with parameter {}, expected {}, got {}",
                a, equiv_result, abstr_result
            );
        }
    }
}

pub(super) fn exec_bi_check<const L: u32, const X: u32>(
    abstr_func: fn(ThreeValuedBitvector<L>, ThreeValuedBitvector<L>) -> ThreeValuedBitvector<X>,
    concr_func: fn(ConcreteBitvector<L>, ConcreteBitvector<L>) -> ConcreteBitvector<X>,
    exact: bool,
) {
    for a in ThreeValuedBitvector::<L>::all_with_length_iter() {
        for b in ThreeValuedBitvector::<L>::all_with_length_iter() {
            let abstr_result = abstr_func(a, b);

            let a_concr_iter =
                ConcreteBitvector::<L>::all_with_length_iter().filter(|c| a.contains_concr(c));
            let equiv_result = join_concr_iter(a_concr_iter.flat_map(|a_concr| {
                ConcreteBitvector::<L>::all_with_length_iter()
                    .filter(|c| b.contains_concr(c))
                    .map(move |b_concr| concr_func(a_concr, b_concr))
            }));

            if exact {
                if !abstr_result.meta_eq(&equiv_result) {
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

pub(super) fn exec_divrem_check<const L: u32, const X: u32>(
    abstr_func: fn(
        ThreeValuedBitvector<L>,
        ThreeValuedBitvector<L>,
    ) -> PanicResult<ThreeValuedBitvector<X>>,
    concr_func: fn(
        ConcreteBitvector<L>,
        ConcreteBitvector<L>,
    ) -> concr::PanicResult<ConcreteBitvector<X>>,
) {
    for a in ThreeValuedBitvector::<L>::all_with_length_iter() {
        for b in ThreeValuedBitvector::<L>::all_with_length_iter() {
            let abstr_panic_result = abstr_func(a, b);
            let abstr_result = abstr_panic_result.result;
            let abstr_panic = abstr_panic_result.panic;

            let a_concr_iter =
                ConcreteBitvector::<L>::all_with_length_iter().filter(|c| a.contains_concr(c));

            let equiv_result = join_concr_iter(a_concr_iter.flat_map(|a_concr| {
                ConcreteBitvector::<L>::all_with_length_iter()
                    .filter(|c| b.contains_concr(c))
                    .map(move |b_concr| concr_func(a_concr, b_concr).result)
            }));

            let a_concr_iter =
                ConcreteBitvector::<L>::all_with_length_iter().filter(|c| a.contains_concr(c));
            let equiv_panic = join_panic_concr_iter(a_concr_iter.flat_map(|a_concr| {
                ConcreteBitvector::<L>::all_with_length_iter()
                    .filter(|c| b.contains_concr(c))
                    .map(move |b_concr| concr_func(a_concr, b_concr).panic)
            }));

            if !abstr_result.contains(&equiv_result) {
                panic!(
                    "Unsound result with parameters {}, {}, expected {}, got {}",
                    a, b, equiv_result, abstr_result
                );
            }
            if !abstr_panic.meta_eq(&equiv_panic) {
                panic!(
                    "Non-exact panic with parameters {}, {}, expected {}, got {}",
                    a, b, equiv_panic, abstr_panic
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

pub(super) fn join_concr_iter<const L: u32>(
    mut iter: impl Iterator<Item = ConcreteBitvector<L>>,
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

pub(super) fn join_bool_concr_iter(iter: impl Iterator<Item = concr::Boolean>) -> Boolean {
    let mut can_be_false = false;
    let mut can_be_true = false;

    for value in iter {
        if value.into_bool() {
            can_be_true = true;
        } else {
            can_be_false = true;
        }
    }

    Boolean::from_bools(can_be_false, can_be_true)
}

pub(super) fn join_panic_concr_iter(
    mut iter: impl Iterator<Item = ConcreteBitvector<32>>,
) -> PanicBitvector {
    let first_concrete = iter
        .next()
        .expect("Expected at least one concrete bitvector in iterator");

    let mut result = PanicBitvector::from_concrete(first_concrete);

    for c in iter {
        result = result.join(PanicBitvector::from_concrete(c))
    }
    result
}
