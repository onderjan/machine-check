use crate::{
    abstr::{Abstr, CBitvector, PanicBitvector, PanicResult},
    bitvector::abstr::three_valued::CThreeValuedBitvector,
    concr::{self, CConcreteBitvector},
    misc::{CBound, Join},
    traits::misc::MetaEq,
};

macro_rules! uni_op_test {
    ($op:tt) => {
        seq_macro::seq!(L in 0..=8 {

        #[test]
        pub fn $op~L() {
            use $crate::abstr::three_valued::CThreeValuedBitvector;
            use $crate::concr::CConcreteBitvector;
            let abstr_func = |a: CThreeValuedBitvector<L>| a.$op();
            let concr_func = |a: CConcreteBitvector<L>| a.$op();
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
                    use $crate::abstr::three_valued::CThreeValuedBitvector;
                    use $crate::concr::CConcreteBitvector;
                    let abstr_func =
                        |a: CThreeValuedBitvector<L>| -> CThreeValuedBitvector<X> { $crate::forward::BExt::$op(a, CBound) };
                    let concr_func = |a: CConcreteBitvector<L>| -> CConcreteBitvector<X> { $crate::forward::BExt::$op(a, CBound) };
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
            use $crate::abstr::three_valued::CThreeValuedBitvector;
            use $crate::concr::CConcreteBitvector;
            use $crate::concr::BoolConvert;
            let abstr_func = |a: CThreeValuedBitvector<L>, b: CThreeValuedBitvector<L>| BoolConvert::bool_into(a.$op(b));
            let concr_func = |a: CConcreteBitvector<L>, b: CConcreteBitvector<L>| BoolConvert::bool_into(a.$op(b));
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
            use $crate::abstr::three_valued::CThreeValuedBitvector;
            use $crate::concr::CConcreteBitvector;
            let abstr_func = |a: CThreeValuedBitvector<L>, b: CThreeValuedBitvector<L>| a.$op(b).into();
            let concr_func = |a: CConcreteBitvector<L>, b: CConcreteBitvector<L>| a.$op(b).into();
            $crate::bitvector::abstr::three_valued::tests::op::exec_divrem_check(abstr_func, concr_func);
        }
    });
    };
}

pub(super) fn exec_uni_check<const W: u32, const X: u32>(
    abstr_func: fn(CThreeValuedBitvector<W>) -> CThreeValuedBitvector<X>,
    concr_func: fn(CConcreteBitvector<W>) -> CConcreteBitvector<X>,
) {
    for a in CThreeValuedBitvector::<W>::all_with_bound_iter(CBound) {
        let abstr_result = abstr_func(a);
        let equiv_result = join_concr_iter(
            CConcreteBitvector::<W>::all_with_bound_iter(CBound)
                .filter(|c| a.contains_concrete(c))
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

pub(super) fn exec_bi_check<const W: u32, const X: u32>(
    abstr_func: fn(CThreeValuedBitvector<W>, CThreeValuedBitvector<W>) -> CThreeValuedBitvector<X>,
    concr_func: fn(CConcreteBitvector<W>, CConcreteBitvector<W>) -> CConcreteBitvector<X>,
    exact: bool,
) {
    for a in CThreeValuedBitvector::<W>::all_with_bound_iter(CBound) {
        for b in CThreeValuedBitvector::<W>::all_with_bound_iter(CBound) {
            let abstr_result = abstr_func(a, b);

            let a_concr_iter = CConcreteBitvector::<W>::all_with_bound_iter(CBound)
                .filter(|c| a.contains_concrete(c));
            let equiv_result = join_concr_iter(a_concr_iter.flat_map(|a_concr| {
                CConcreteBitvector::<W>::all_with_bound_iter(CBound)
                    .filter(|c| b.contains_concrete(c))
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

pub(super) fn exec_divrem_check<const W: u32, const X: u32>(
    abstr_func: fn(
        CThreeValuedBitvector<W>,
        CThreeValuedBitvector<W>,
    ) -> PanicResult<CThreeValuedBitvector<X>>,
    concr_func: fn(
        CConcreteBitvector<W>,
        CConcreteBitvector<W>,
    ) -> concr::PanicResult<CConcreteBitvector<X>>,
) {
    for a in CThreeValuedBitvector::<W>::all_with_bound_iter(CBound) {
        for b in CThreeValuedBitvector::<W>::all_with_bound_iter(CBound) {
            let abstr_panic_result = abstr_func(a, b);
            let abstr_result = abstr_panic_result.result;
            let abstr_panic = abstr_panic_result.panic;

            let a_concr_iter = CConcreteBitvector::<W>::all_with_bound_iter(CBound)
                .filter(|c| a.contains_concrete(c));

            let equiv_result = join_concr_iter(a_concr_iter.flat_map(|a_concr| {
                CConcreteBitvector::<W>::all_with_bound_iter(CBound)
                    .filter(|c| b.contains_concrete(c))
                    .map(move |b_concr| concr_func(a_concr, b_concr).result)
            }));

            let a_concr_iter = CConcreteBitvector::<W>::all_with_bound_iter(CBound)
                .filter(|c| a.contains_concrete(c));
            let equiv_panic = join_panic_concr_iter(a_concr_iter.flat_map(|a_concr| {
                CConcreteBitvector::<W>::all_with_bound_iter(CBound)
                    .filter(|c| b.contains_concrete(c))
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
) -> CThreeValuedBitvector<W> {
    if W == 0 {
        return CThreeValuedBitvector::new_unknown(CBound);
    }

    let first_concrete = iter
        .next()
        .expect("Expected at least one concrete bitvector in iterator");

    let mut result = CThreeValuedBitvector::from_concrete_value(first_concrete);

    for c in iter {
        let abstr = CBitvector::from_concrete_value(c);

        result = result.join(&abstr);
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
