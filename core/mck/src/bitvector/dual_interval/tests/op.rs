use crate::{
    abstr::{Abstr, Bitvector, PanicResult},
    bitvector::{
        concrete::{ConcreteBitvector, SignlessInterval},
        dual_interval::DualInterval,
    },
    boolean::abstr,
    concr::{self, Test},
    traits::misc::MetaEq,
};

macro_rules! uni_op_test {
    ($op:tt) => {
        seq_macro::seq!(L in 0..=6 {

        #[test]
        pub fn $op~L() {
            let abstr_func = |a: DualInterval<L>| a.$op();
            let concr_func = |a: ConcreteBitvector<L>| a.$op();
            $crate::bitvector::dual_interval::tests::op::exec_uni_check(abstr_func, concr_func, true);
        }
    });
    };
}

macro_rules! ext_op_test {
    ($op:tt, $exact:tt) => {
        seq_macro::seq!(L in 0..=6 {
            seq_macro::seq!(X in 0..=6 {
                #[test]
                pub fn $op~L~X() {
                    let abstr_func =
                        |a: DualInterval<L>| -> DualInterval<X> { a.$op() };
                    let concr_func = |a: ConcreteBitvector<L>| -> ConcreteBitvector<X> { a.$op() };
                    $crate::bitvector::dual_interval::tests::op::exec_uni_check(abstr_func, concr_func, $exact);
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
            let abstr_func = |a: DualInterval<L>, b: DualInterval<L>| ::std::convert::Into::into(a.$op(b));
            let concr_func = |a: ConcreteBitvector<L>, b: ConcreteBitvector<L>|  ::std::convert::Into::into(a.$op(b));
            $crate::bitvector::dual_interval::tests::op::exec_bi_check(abstr_func, concr_func, $exact);
        }
    });
    };
}

macro_rules! comparison_op_test {
    ($op:tt,$exact:tt) => {

        seq_macro::seq!(L in 0..=4 {

        #[test]
        pub fn $op~L() {
            let abstr_func = |a: DualInterval<L>, b: DualInterval<L>| ::std::convert::Into::into(a.$op(b));
            let concr_func = |a: ConcreteBitvector<L>, b: ConcreteBitvector<L>|  ::std::convert::Into::into(a.$op(b));
            $crate::bitvector::dual_interval::tests::op::exec_comparison_check(abstr_func, concr_func, $exact);
        }
    });
    };
}

macro_rules! divrem_op_test {
    ($op:tt,$exact:tt) => {

        seq_macro::seq!(L in 0..=4 {

        #[test]
        pub fn $op~L() {
            let abstr_func = |a: DualInterval<L>, b: DualInterval<L>| a.$op(b).into();
            let concr_func = |a: ConcreteBitvector<L>, b: ConcreteBitvector<L>| a.$op(b).into();
            $crate::bitvector::dual_interval::tests::op::exec_divrem_check(abstr_func, concr_func);
        }
    });
    };
}

pub(super) fn exec_uni_check<const L: u32, const X: u32>(
    abstr_func: fn(DualInterval<L>) -> DualInterval<X>,
    concr_func: fn(ConcreteBitvector<L>) -> ConcreteBitvector<X>,
    exact: bool,
) {
    for a in DualInterval::<L>::all_with_length_iter() {
        let abstr_result = abstr_func(a);
        let equiv_result = join_concr_iter(
            ConcreteBitvector::<L>::all_with_length_iter()
                .filter(|c| a.contains_value(c))
                .map(concr_func),
        );
        if exact {
            if !abstr_result.meta_eq(&equiv_result) {
                panic!(
                    "Non-exact result with parameter {}, expected {}, got {}",
                    a, equiv_result, abstr_result
                );
            }
        } else if !abstr_result.contains(&equiv_result) {
            panic!(
                "Unsound result with parameter {}, expected {}, got {}",
                a, equiv_result, abstr_result
            );
        }
    }
}

pub(super) fn exec_bi_check<const L: u32, const X: u32>(
    abstr_func: fn(DualInterval<L>, DualInterval<L>) -> DualInterval<X>,
    concr_func: fn(ConcreteBitvector<L>, ConcreteBitvector<L>) -> ConcreteBitvector<X>,
    exact: bool,
) {
    for a in DualInterval::<L>::all_with_length_iter() {
        for b in DualInterval::<L>::all_with_length_iter() {
            let abstr_result = abstr_func(a, b);

            let a_concr_iter =
                ConcreteBitvector::<L>::all_with_length_iter().filter(|c| a.contains_value(c));
            let equiv_result = join_concr_iter(a_concr_iter.flat_map(|a_concr| {
                ConcreteBitvector::<L>::all_with_length_iter()
                    .filter(|c| b.contains_value(c))
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

pub(super) fn exec_comparison_check<const L: u32>(
    abstr_func: fn(DualInterval<L>, DualInterval<L>) -> abstr::Boolean,
    concr_func: fn(ConcreteBitvector<L>, ConcreteBitvector<L>) -> concr::Boolean,
    exact: bool,
) {
    for a in DualInterval::<L>::all_with_length_iter() {
        for b in DualInterval::<L>::all_with_length_iter() {
            let abstr_result: abstr::Boolean = abstr_func(a, b);

            let a_concr_iter =
                ConcreteBitvector::<L>::all_with_length_iter().filter(|c| a.contains_value(c));
            let equiv_result: abstr::Boolean =
                join_bool_concr_iter(a_concr_iter.flat_map(|a_concr| {
                    ConcreteBitvector::<L>::all_with_length_iter()
                        .filter(|c| b.contains_value(c))
                        .map(move |b_concr| concr_func(a_concr, b_concr))
                }));

            if exact {
                if !abstr_result.0.meta_eq(&equiv_result.0) {
                    panic!(
                        "Non-exact result with parameters {}, {}, expected {}, got {}",
                        a, b, equiv_result, abstr_result
                    );
                }
            } else if !abstr_result.0.contains(&equiv_result.0) {
                panic!(
                    "Unsound result with parameters {}, {}, expected {}, got {}",
                    a, b, equiv_result, abstr_result
                );
            }
            if a.concrete_value().is_some()
                && b.concrete_value().is_some()
                && abstr_result.0.concrete_value().is_none()
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
    abstr_func: fn(DualInterval<L>, DualInterval<L>) -> PanicResult<DualInterval<X>>,
    concr_func: fn(
        ConcreteBitvector<L>,
        ConcreteBitvector<L>,
    ) -> concr::PanicResult<ConcreteBitvector<X>>,
) {
    for a in DualInterval::<L>::all_with_length_iter() {
        for b in DualInterval::<L>::all_with_length_iter() {
            let abstr_panic_result = abstr_func(a, b);
            let abstr_result = abstr_panic_result.result;
            let abstr_panic = abstr_panic_result.panic;

            let a_concr_iter =
                ConcreteBitvector::<L>::all_with_length_iter().filter(|c| a.contains_value(c));

            let equiv_result = join_concr_iter(a_concr_iter.flat_map(|a_concr| {
                ConcreteBitvector::<L>::all_with_length_iter()
                    .filter(|c| b.contains_value(c))
                    .map(move |b_concr| concr_func(a_concr, b_concr).result)
            }));

            let a_concr_iter =
                ConcreteBitvector::<L>::all_with_length_iter().filter(|c| a.contains_value(c));
            let equiv_panic = join_tvbv_concr_iter(a_concr_iter.flat_map(|a_concr| {
                ConcreteBitvector::<L>::all_with_length_iter()
                    .filter(|c| b.contains_value(c))
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
) -> DualInterval<L> {
    if L == 0 {
        return DualInterval::FULL;
    }

    let first_concrete = iter
        .next()
        .expect("Expected at least one concrete bitvector in iterator");

    let mut result = DualInterval::from_value(first_concrete);

    for c in iter {
        result = result.concrete_join(c)
    }
    result
}

pub(super) fn join_tvbv_concr_iter<const L: u32>(
    mut iter: impl Iterator<Item = ConcreteBitvector<L>>,
) -> Bitvector<L> {
    if L == 0 {
        return Bitvector::new_unknown();
    }

    let first_concrete = iter
        .next()
        .expect("Expected at least one concrete bitvector in iterator");

    let mut result = Bitvector::from_concrete(first_concrete);

    for c in iter {
        result = result.concrete_join(c)
    }
    result
}

pub(super) fn join_bool_concr_iter(iter: impl Iterator<Item = concr::Boolean>) -> abstr::Boolean {
    let mut can_be_false = false;
    let mut can_be_true = false;

    for value in iter {
        if value.into_bool() {
            can_be_true = true;
        } else {
            can_be_false = true;
        }
    }

    abstr::Boolean::from_bools(can_be_false, can_be_true)
}

impl<const W: u32> DualInterval<W> {
    pub fn contains(&self, other: &Self) -> bool {
        if other.near_half == other.far_half {
            let tested_half = other.near_half;
            if tested_half.is_sign_bit_set() {
                self.far_half.contains(&other.far_half)
            } else {
                self.near_half.contains(&other.near_half)
            }
        } else {
            self.near_half.contains(&other.near_half) && self.far_half.contains(&other.far_half)
        }
    }

    pub fn concrete_join(self, value: ConcreteBitvector<W>) -> Self {
        let value_sign_bit_set = value.is_sign_bit_set();
        let value = SignlessInterval::from_value(value);

        if self.near_half == self.far_half {
            if value_sign_bit_set == self.near_half.is_sign_bit_set() {
                // join to both halves
                Self {
                    near_half: self.near_half.union(value),
                    far_half: self.far_half.union(value),
                }
            } else {
                // we have to make a new half from the value
                if value_sign_bit_set {
                    Self {
                        near_half: self.near_half,
                        far_half: value,
                    }
                } else {
                    Self {
                        near_half: value,
                        far_half: self.far_half,
                    }
                }
            }
        } else if value_sign_bit_set {
            // join to far half
            Self {
                near_half: self.near_half,
                far_half: self.far_half.union(value),
            }
        } else {
            // join to near half
            Self {
                near_half: self.near_half.union(value),
                far_half: self.far_half,
            }
        }
    }

    pub fn all_with_length_iter() -> impl Iterator<Item = Self> {
        let only_near_half_result = SignlessInterval::all_with_length_iter(false)
            .map(|near_half| Self::from_opt_halves(Some(near_half), None));
        let only_far_half_result = SignlessInterval::all_with_length_iter(true)
            .map(|far_half| Self::from_opt_halves(None, Some(far_half)));

        let near_half_iter = SignlessInterval::<W>::all_with_length_iter(false);
        let both_halves_result = near_half_iter.flat_map(|near_half| {
            let far_half_iter = SignlessInterval::<W>::all_with_length_iter(true);
            far_half_iter
                .map(move |far_half| Self::from_opt_halves(Some(near_half), Some(far_half)))
        });
        only_near_half_result
            .chain(only_far_half_result)
            .chain(both_halves_result)
    }
}
