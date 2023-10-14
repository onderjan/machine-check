use crate::bitvector::{concr, three_valued::abstr::ThreeValuedBitvector};

macro_rules! uni_op_test {
    ($op:tt) => {
        seq_macro::seq!(L in 0..=8 {

        #[test]
        pub fn $op~L() {
            let abstr_func = |a: ThreeValuedBitvector<L>| a.$op();
            let concr_func = |a: concr::Bitvector<L>| a.$op();
            $crate::bitvector::three_valued::abstr::test::op::exec_uni_check(abstr_func, concr_func);
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
                    $crate::bitvector::three_valued::abstr::test::op::exec_uni_check(abstr_func, concr_func);
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
            $crate::bitvector::three_valued::abstr::test::op::exec_bi_check(abstr_func, concr_func, $exact);
        }
    });
    };
}

pub(super) fn exec_uni_check<const L: u32, const X: u32>(
    abstr_func: fn(ThreeValuedBitvector<L>) -> ThreeValuedBitvector<X>,
    concr_func: fn(concr::Bitvector<L>) -> concr::Bitvector<X>,
) {
    for a in ThreeValuedBitvector::<L>::all_with_length_iter() {
        let abstr_result = abstr_func(a);
        let equiv_result = join_concr_iter(
            concr::Bitvector::<L>::all_with_length_iter()
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

pub(super) fn exec_bi_check<const L: u32, const X: u32>(
    abstr_func: fn(ThreeValuedBitvector<L>, ThreeValuedBitvector<L>) -> ThreeValuedBitvector<X>,
    concr_func: fn(concr::Bitvector<L>, concr::Bitvector<L>) -> concr::Bitvector<X>,
    exact: bool,
) {
    for a in ThreeValuedBitvector::<L>::all_with_length_iter() {
        for b in ThreeValuedBitvector::<L>::all_with_length_iter() {
            let abstr_result = abstr_func(a, b);

            let a_concr_iter =
                concr::Bitvector::<L>::all_with_length_iter().filter(|c| a.contains_concr(c));
            let equiv_result = join_concr_iter(a_concr_iter.flat_map(|a_concr| {
                concr::Bitvector::<L>::all_with_length_iter()
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

pub(super) fn join_concr_iter<const L: u32>(
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
