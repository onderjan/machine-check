use std::collections::BTreeSet;

use super::Bitvector;
use crate::{bitvector::concrete::ConcreteBitvector, forward::HwArith};

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
            let abstr_func = |a: Bitvector<L>, b: Bitvector<L>| a.$op(b);
            let concr_func = |a: ConcreteBitvector<L>, b: ConcreteBitvector<L>| a.$op(b);
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
        let equiv_result = join_concr_iter(a.concrete_iter().map(concr_func));
        if abstr_result != equiv_result {
            panic!(
                "Wrong result with parameter {}, expected {}, got {}",
                a, equiv_result, abstr_result
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

            let equiv_result = join_concr_iter(a.concrete_iter().flat_map(|a_concr| {
                b.concrete_iter()
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
    iter: impl Iterator<Item = ConcreteBitvector<L>>,
) -> Bitvector<L> {
    if L == 0 {
        return Bitvector::full();
    }

    let set: BTreeSet<_> = iter.map(|a| a.as_unsigned()).collect();
    assert!(!set.is_empty());

    // the optimal wrapping interval contains the largest hole between subsequent elements
    // make sure the wrapping between the last element and first element is included
    let mut largest_hole = 0;
    let mut largest_hole_index = 0;
    for (index, (current, next)) in set
        .iter()
        .cloned()
        .zip(set.iter().skip(1).cloned())
        .chain(std::iter::once((
            *set.last().unwrap(),
            *set.first().unwrap(),
        )))
        .enumerate()
    {
        let current_bitvec = ConcreteBitvector::<L>::new(current);
        let next_bitvec = ConcreteBitvector::<L>::new(next);
        let hole = next_bitvec.sub(current_bitvec).as_unsigned();
        if hole > largest_hole {
            largest_hole = hole;
            largest_hole_index = index;
        }
    }

    // construct the result
    let start_index = if largest_hole_index + 1 < set.len() {
        largest_hole_index + 1
    } else {
        0
    };
    let end_index = largest_hole_index;
    let start = ConcreteBitvector::new(*set.iter().nth(start_index).unwrap());
    let end = ConcreteBitvector::new(*set.iter().nth(end_index).unwrap());

    let result = Bitvector::from_wrap_interval(start, end);
    result
}
