use std::collections::BTreeSet;

use super::Bitvector;
use crate::{
    abstr::{Boolean, Test},
    bitvector::concrete::ConcreteBitvector,
    forward::HwArith,
};

macro_rules! uni_op_test {
    ($op:tt) => {
        seq_macro::seq!(L in 0..=8 {

        #[test]
        pub fn $op~L() {
            let abstr_func = |a: Bitvector<L>| a.$op();
            let concr_func = |a: ConcreteBitvector<L>| a.$op();
            $crate::bitvector::wrap_interval::abstr::tests::op::exec_uni_check(abstr_func, concr_func);
        }
    });
    };
}

macro_rules! ext_op_test {
    ($op:tt) => {
        seq_macro::seq!(L in 0..=4 {
            seq_macro::seq!(X in 0..=4 {
                #[test]
                pub fn $op~L~X() {
                    let abstr_func =
                        |a: Bitvector<L>| -> Bitvector<X> { a.$op() };
                    let concr_func = |a: ConcreteBitvector<L>| -> ConcreteBitvector<X> { a.$op() };
                    $crate::bitvector::wrap_interval::abstr::tests::op::exec_uni_check(abstr_func, concr_func);
                }
            });
        });
    };
}

macro_rules! bi_op_test {
    ($op:tt,$exact:tt) => {

        seq_macro::seq!(L in 0..=4 {

        #[test]
        pub fn $op~L() {
            let abstr_func = |a: Bitvector<L>, b: Bitvector<L>| ::std::convert::Into::into(a.$op(b));
            let concr_func = |a: ConcreteBitvector<L>, b: ConcreteBitvector<L>| ::std::convert::Into::into(a.$op(b));
            $crate::bitvector::wrap_interval::abstr::tests::op::exec_bi_check(abstr_func, concr_func, $exact);
        }
    });
    };
}

pub(super) fn exec_uni_check<const L: u32, const X: u32>(
    abstr_func: fn(Bitvector<L>) -> Bitvector<X>,
    concr_func: fn(ConcreteBitvector<L>) -> ConcreteBitvector<X>,
) {
    for a in Bitvector::<L>::all_with_length_iter() {
        let abstr_result = abstr_func(a);

        let concrete_set: BTreeSet<u64> = a
            .concrete_iter()
            .map(|v| concr_func(v).as_unsigned())
            .collect();

        for concrete in concrete_set.iter().cloned() {
            if !abstr_result.contains_concrete(&ConcreteBitvector::new(concrete)) {
                panic!(
                    "Unsound result with parameter {}, expected {:?}, got {}",
                    a, concrete_set, abstr_result
                );
            }
        }

        let expected_largest_hole = largest_hole::<X>(&concrete_set);
        let abstr_largest_hole = abstr_result.hole_diff().as_unsigned();

        if abstr_largest_hole != expected_largest_hole {
            panic!(
                        "Non-exact result with parameter, {}, expected {:?} (largest hole {}), got {} (hole {})",
                        a, concrete_set, expected_largest_hole, abstr_result, abstr_largest_hole
                    );
        }
    }
}

pub(super) fn exec_bi_check<const L: u32, const X: u32>(
    abstr_func: fn(Bitvector<L>, Bitvector<L>) -> Bitvector<X>,
    concr_func: fn(ConcreteBitvector<L>, ConcreteBitvector<L>) -> ConcreteBitvector<X>,
    exact: bool,
) {
    for a in Bitvector::<L>::all_with_length_iter() {
        for b in Bitvector::<L>::all_with_length_iter() {
            let abstr_result = abstr_func(a, b);

            let concrete_iter = a.concrete_iter().flat_map(|a_concr| {
                b.concrete_iter()
                    .map(move |b_concr| concr_func(a_concr, b_concr))
            });
            let concrete_set: BTreeSet<u64> = concrete_iter.map(|v| v.as_unsigned()).collect();

            for concrete in concrete_set.iter().cloned() {
                if !abstr_result.contains_concrete(&ConcreteBitvector::new(concrete)) {
                    panic!(
                        "Unsound result with parameters {}, {}, expected {:?}, got {}",
                        a, b, concrete_set, abstr_result
                    );
                }
            }

            if exact {
                let expected_largest_hole = largest_hole::<X>(&concrete_set);
                let abstr_largest_hole = abstr_result.hole_diff().as_unsigned();

                if abstr_largest_hole != expected_largest_hole {
                    panic!(
                            "Non-exact result with parameters {}, {}, expected {:?} (largest hole {}), got {} (hole {})",
                            a, b, concrete_set, expected_largest_hole, abstr_result, abstr_largest_hole
                        );
                }
            }
            if a.concrete_value().is_some()
                && b.concrete_value().is_some()
                && abstr_result.concrete_value().is_none()
            {
                panic!(
                            "Non-concrete-value result with concrete-value parameters {}, {}, expected {:?}, got {}",
                            a, b, concrete_set, abstr_result
                        );
            }
        }
    }
}

pub(super) fn largest_hole<const L: u32>(set: &BTreeSet<u64>) -> u64 {
    if set.is_empty() {
        return 0;
    }

    // the optimal wrapping interval contains the largest hole between subsequent elements
    // make sure the wrapping between the last element and first element is included
    let mut largest_hole = 0;
    for (current, next) in
        set.iter()
            .cloned()
            .zip(set.iter().skip(1).cloned())
            .chain(std::iter::once((
                *set.last().unwrap(),
                *set.first().unwrap(),
            )))
    {
        let current_bitvec = ConcreteBitvector::<L>::new(current);
        let next_bitvec = ConcreteBitvector::<L>::new(next);
        let hole = next_bitvec.sub(current_bitvec).as_unsigned();
        if hole > largest_hole {
            largest_hole = hole;
        }
    }
    largest_hole
}
