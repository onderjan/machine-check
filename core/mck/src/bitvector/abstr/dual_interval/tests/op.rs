use crate::{
    abstr::{Abstr, BitvectorDomain, PanicBitvector, PanicResult},
    bitvector::{abstr::dual_interval::CDualInterval, interval::SignlessInterval},
    boolean::abstr,
    concr::{self, CConcreteBitvector, Test},
    misc::{CBound, Join},
    traits::misc::MetaEq,
};

macro_rules! uni_op_test {
    ($op:tt) => {
        seq_macro::seq!(L in 0..=6 {

        #[test]
        pub fn $op~L() {
            let abstr_func = |a: CDualInterval<L>| a.$op();
            let concr_func = |a: CConcreteBitvector<L>| a.$op();
            $crate::bitvector::abstr::dual_interval::tests::op::exec_uni_check(abstr_func, concr_func, true);
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
                        |a: CDualInterval<L>| -> CDualInterval<X> { Ext::$op(a) };
                    let concr_func = |a: CConcreteBitvector<L>| -> CConcreteBitvector<X> { Ext::$op(a) };
                    $crate::bitvector::abstr::dual_interval::tests::op::exec_uni_check(abstr_func, concr_func, $exact);
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
            let abstr_func = |a: CDualInterval<L>, b: CDualInterval<L>| ::std::convert::Into::into(a.$op(b));
            let concr_func = |a: CConcreteBitvector<L>, b: CConcreteBitvector<L>|  ::std::convert::Into::into(a.$op(b));
            $crate::bitvector::abstr::dual_interval::tests::op::exec_bi_check(abstr_func, concr_func, $exact);
        }
    });
    };
}

macro_rules! comparison_op_test {
    ($op:tt,$exact:tt) => {

        seq_macro::seq!(L in 0..=4 {

        #[test]
        pub fn $op~L() {
            let abstr_func = |a: CDualInterval<L>, b: CDualInterval<L>| ::std::convert::Into::into(a.$op(b));
            let concr_func = |a: CConcreteBitvector<L>, b: CConcreteBitvector<L>|  ::std::convert::Into::into(a.$op(b));
            $crate::bitvector::abstr::dual_interval::tests::op::exec_comparison_check(abstr_func, concr_func, $exact);
        }
    });
    };
}

macro_rules! divrem_op_test {
    ($op:tt,$exact:tt) => {

        seq_macro::seq!(L in 0..=4 {

        #[test]
        pub fn $op~L() {
            let abstr_func = |a: CDualInterval<L>, b: CDualInterval<L>| a.$op(b).into();
            let concr_func = |a: CConcreteBitvector<L>, b: CConcreteBitvector<L>| a.$op(b).into();
            $crate::bitvector::abstr::dual_interval::tests::op::exec_divrem_check(abstr_func, concr_func);
        }
    });
    };
}

pub(super) fn exec_uni_check<const W: u32, const X: u32>(
    abstr_func: fn(CDualInterval<W>) -> CDualInterval<X>,
    concr_func: fn(CConcreteBitvector<W>) -> CConcreteBitvector<X>,
    exact: bool,
) {
    for a in CDualInterval::<W>::all_with_bound_iter() {
        let abstr_result = abstr_func(a);
        let equiv_result = join_concr_iter(
            CConcreteBitvector::<W>::all_with_bound_iter(CBound)
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

pub(super) fn exec_bi_check<const W: u32, const X: u32>(
    abstr_func: fn(CDualInterval<W>, CDualInterval<W>) -> CDualInterval<X>,
    concr_func: fn(CConcreteBitvector<W>, CConcreteBitvector<W>) -> CConcreteBitvector<X>,
    exact: bool,
) {
    for a in CDualInterval::<W>::all_with_bound_iter() {
        for b in CDualInterval::<W>::all_with_bound_iter() {
            let abstr_result = abstr_func(a, b);

            let a_concr_iter = CConcreteBitvector::<W>::all_with_bound_iter(CBound)
                .filter(|c| a.contains_value(c));
            let equiv_result = join_concr_iter(a_concr_iter.flat_map(|a_concr| {
                CConcreteBitvector::<W>::all_with_bound_iter(CBound)
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

pub(super) fn exec_comparison_check<const W: u32>(
    abstr_func: fn(CDualInterval<W>, CDualInterval<W>) -> abstr::Boolean,
    concr_func: fn(CConcreteBitvector<W>, CConcreteBitvector<W>) -> concr::Boolean,
    exact: bool,
) {
    for a in CDualInterval::<W>::all_with_bound_iter() {
        for b in CDualInterval::<W>::all_with_bound_iter() {
            let abstr_result: abstr::Boolean = abstr_func(a, b);

            let a_concr_iter = CConcreteBitvector::<W>::all_with_bound_iter(CBound)
                .filter(|c| a.contains_value(c));
            let equiv_result: abstr::Boolean =
                join_bool_concr_iter(a_concr_iter.flat_map(|a_concr| {
                    CConcreteBitvector::<W>::all_with_bound_iter(CBound)
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
            } else {
                let joined = abstr_result.join(&equiv_result);
                if !abstr_result.meta_eq(&joined) {
                    panic!(
                        "Unsound result with parameters {}, {}, expected {}, got {}",
                        a, b, equiv_result, abstr_result
                    );
                }
            }
            if a.concrete_value().is_some()
                && b.concrete_value().is_some()
                && abstr_result.is_unknown()
            {
                panic!(
                            "Non-concrete-value result with concrete-value parameters {}, {}, expected {}, got {}",
                            a, b, equiv_result, abstr_result
                        );
            }
        }
    }
}

pub(super) fn exec_divrem_check<const W: u32, const X: u32>(
    abstr_func: fn(CDualInterval<W>, CDualInterval<W>) -> PanicResult<CDualInterval<X>>,
    concr_func: fn(
        CConcreteBitvector<W>,
        CConcreteBitvector<W>,
    ) -> concr::PanicResult<CConcreteBitvector<X>>,
) {
    for a in CDualInterval::<W>::all_with_bound_iter() {
        for b in CDualInterval::<W>::all_with_bound_iter() {
            let abstr_panic_result = abstr_func(a, b);
            let abstr_result = abstr_panic_result.result;
            let abstr_panic = abstr_panic_result.panic;

            let a_concr_iter = CConcreteBitvector::<W>::all_with_bound_iter(CBound)
                .filter(|c| a.contains_value(c));

            let equiv_result = join_concr_iter(a_concr_iter.flat_map(|a_concr| {
                CConcreteBitvector::<W>::all_with_bound_iter(CBound)
                    .filter(|c| b.contains_value(c))
                    .map(move |b_concr| concr_func(a_concr, b_concr).result)
            }));

            let a_concr_iter = CConcreteBitvector::<W>::all_with_bound_iter(CBound)
                .filter(|c| a.contains_value(c));
            let equiv_panic = join_panic_concr_iter(a_concr_iter.flat_map(|a_concr| {
                CConcreteBitvector::<W>::all_with_bound_iter(CBound)
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

pub(super) fn join_concr_iter<const W: u32>(
    mut iter: impl Iterator<Item = CConcreteBitvector<W>>,
) -> CDualInterval<W> {
    if W == 0 {
        return CDualInterval::top(CBound);
    }

    let first_concrete = iter
        .next()
        .expect("Expected at least one concrete bitvector in iterator");

    let mut result = CDualInterval::from_value(first_concrete);

    for c in iter {
        result = result.concrete_join(c)
    }
    result
}

pub(super) fn join_panic_concr_iter(
    mut iter: impl Iterator<Item = CConcreteBitvector<32>>,
) -> PanicBitvector {
    let first_concrete = iter
        .next()
        .expect("Expected at least one concrete bitvector in iterator");

    let mut result = PanicBitvector::from_concrete(first_concrete);

    for c in iter {
        result = result.join(&PanicBitvector::from_concrete(c))
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

impl<const W: u32> CDualInterval<W> {
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

    pub fn concrete_join(self, value: CConcreteBitvector<W>) -> Self {
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

    pub fn all_with_bound_iter() -> impl Iterator<Item = Self> {
        let only_near_half_result = SignlessInterval::all_with_bound_iter(CBound, false)
            .map(|near_half| Self::from_opt_halves(Some(near_half), None));
        let only_far_half_result = SignlessInterval::all_with_bound_iter(CBound, true)
            .map(|far_half| Self::from_opt_halves(None, Some(far_half)));

        let near_half_iter = SignlessInterval::all_with_bound_iter(CBound, false);
        let both_halves_result = near_half_iter.flat_map(|near_half| {
            let far_half_iter = SignlessInterval::all_with_bound_iter(CBound, true);
            far_half_iter
                .map(move |far_half| Self::from_opt_halves(Some(near_half), Some(far_half)))
        });
        only_near_half_result
            .chain(only_far_half_result)
            .chain(both_halves_result)
    }
}
