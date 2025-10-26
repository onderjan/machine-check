use crate::{
    bitvector::{abstr::three_valued::RThreeValuedBitvector, refin::three_valued::RMarkBitvector},
    boolean::concr,
    concr::RConcreteBitvector,
    misc::{MetaEq, RBound},
    refin::RefinementDomain,
};

macro_rules! uni_op_test {
    ($ty:tt, $op:tt, $exact:tt) => {

        seq_macro::seq!(L in 0..=6 {

        #[test]
        pub fn $op~L() {
            let mark_func = |a,
                                a_mark: RMarkBitvector|
                -> RMarkBitvector { $crate::backward::$ty::$op((a,), a_mark).0 };
            let concr_func = $crate::forward::$ty::$op;
            $crate::bitvector::refin::three_valued::tests::op::exec_uni_check(mark_func, concr_func, L, L, $exact);
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
                let mark_func = |a,
                                a_mark: RMarkBitvector|
                -> RMarkBitvector { $crate::backward::$ty::$op((a,), a_mark).0 };
                let concr_func = |a| { $crate::forward::BExt::$op(a, RBound::new(X)) };
                $crate::bitvector::refin::three_valued::tests::op::exec_uni_check(mark_func, concr_func, L, X, $exact);
            }
            });
        });
    };
}

macro_rules! bi_op_test {
    ($ty:tt, $op:tt, $exact:tt) => {

        seq_macro::seq!(L in 0..=3 {

        #[test]
        pub fn $op~L() {
            let mark_func = |inputs: (RThreeValuedBitvector, RThreeValuedBitvector),
                                mark| { crate::backward::$ty::$op(inputs, mark) };
            let concr_func = |a: RConcreteBitvector, b: RConcreteBitvector| -> RConcreteBitvector {
                $crate::forward::$ty::$op(a,b)
            };

            $crate::bitvector::refin::three_valued::tests::op::exec_bi_check(&mark_func, &concr_func, L, $exact);
        }
    });
    };
}

macro_rules! cmp_op_test {
    ($ty:tt, $op:tt, $exact:tt) => {

        seq_macro::seq!(L in 0..=3 {

        #[test]
        pub fn $op~L() {
            let mark_func = |inputs: (RThreeValuedBitvector, RThreeValuedBitvector),
                                mark| { crate::backward::$ty::$op(inputs, mark) };
            let concr_func = |a: RConcreteBitvector, b: RConcreteBitvector| {
                $crate::forward::$ty::$op(a,b)
            };

            $crate::bitvector::refin::three_valued::tests::op::exec_cmp_check(&mark_func, &concr_func, L, $exact);
        }
    });
    };
}

macro_rules! divrem_op_test {
    ($ty:tt, $op:tt, $exact:tt) => {

        seq_macro::seq!(L in 0..=3 {

        #[test]
        pub fn $op~L() {
            let mark_func = |inputs: (RThreeValuedBitvector, RThreeValuedBitvector),
                                mark| {
                                    crate::backward::$ty::$op(inputs, mark)
            };
            let concr_func = |a: RConcreteBitvector, b: RConcreteBitvector| {
                let panic_result = $crate::forward::$ty::$op(a,b);
                (panic_result.result, panic_result.panic.as_runtime_bitvector())
            };
            $crate::bitvector::refin::three_valued::tests::op::exec_divrem_check(&mark_func, &concr_func, L, $exact);
        }
    });
    };
}

fn exact_uni_mark(
    a_abstr: RThreeValuedBitvector,
    a_mark: RMarkBitvector,
    input_width: u32,
    output_width: u32,
    concr_func: fn(RConcreteBitvector) -> RConcreteBitvector,
) -> RMarkBitvector {
    // the result marks exactly those bits of input which, if changed in operation input,
    // can change bits masked by mark_a in the operation result
    let mark_mask = a_mark.marked_bits().to_u64();
    // determine for each input bit separately
    let mut result = 0;
    for i in 0..input_width {
        for a in RConcreteBitvector::all_with_bound_iter(RBound::new(input_width)) {
            if a.to_u64() & (1 << i) != 0 {
                continue;
            }
            let with_zero = a;
            let with_one = RConcreteBitvector::new(a.to_u64() | (1 << i), RBound::new(input_width));
            if !a_abstr.contains_concrete(&with_zero) || !a_abstr.contains_concrete(&with_one) {
                continue;
            }
            if concr_func(with_zero).to_u64() & mark_mask
                != concr_func(with_one).to_u64() & mark_mask
            {
                result |= 1 << i;
            }
        }
    }
    RMarkBitvector::new_from_flag(RConcreteBitvector::new(result, RBound::new(output_width)))
}

fn eval_mark(
    want_exact: bool,
    exact_earlier: RMarkBitvector,
    our_earlier: RMarkBitvector,
    provoked: bool,
) {
    if want_exact {
        // test for exactness
        if exact_earlier != our_earlier {
            panic!(
                "Non-exact, expected {}, got {}",
                exact_earlier.marked_bits(),
                our_earlier.marked_bits()
            );
        }
    } else {
        // test whether our earlier mark is at least as marked as the exact one
        // if not, the marking will be incomplete
        let exact = exact_earlier.marked_bits().to_u64();
        let our = our_earlier.marked_bits().to_u64();
        if our & exact != exact {
            panic!(
                "Incomplete, expected {}, got {}",
                exact_earlier.marked_bits(),
                our_earlier.marked_bits()
            );
        }
        // test unprovoked marking
        if !provoked && our_earlier.marked_bits().is_nonzero() {
            panic!(
                "Unprovoked, expected {}, got {}",
                exact_earlier.marked_bits(),
                our_earlier.marked_bits()
            );
        }
    }
}

pub(super) fn exec_uni_check(
    mark_func: fn(RThreeValuedBitvector, RMarkBitvector) -> RMarkBitvector,
    concr_func: fn(RConcreteBitvector) -> RConcreteBitvector,
    input_width: u32,
    output_width: u32,
    want_exact: bool,
) {
    // a mark bit is necessary if changing the input bit can impact the output
    // test this for all concretizations of the input

    for a_later in RConcreteBitvector::all_with_bound_iter(RBound::new(output_width)) {
        let a_later = RMarkBitvector::new_from_flag(a_later);

        for a_abstr in RThreeValuedBitvector::all_with_bound_iter(RBound::new(input_width)) {
            let exact_earlier =
                exact_uni_mark(a_abstr, a_later, input_width, output_width, concr_func);
            let our_earlier = mark_func(a_abstr, a_later);
            eval_mark(
                want_exact,
                exact_earlier,
                our_earlier,
                a_later.importance() > 0,
            );
        }
    }
}

fn exact_left_mark(
    abstr: (RThreeValuedBitvector, RThreeValuedBitvector),
    mark: RMarkBitvector,
    concr_func: impl Fn(RConcreteBitvector, RConcreteBitvector) -> RConcreteBitvector,
    input_width: u32,
    output_width: u32,
) -> RMarkBitvector {
    let left_abstr = abstr.0;
    let right_abstr = abstr.1;
    // the result marks exactly those bits of input which, if changed in operation input,
    // can change bits masked by mark_a in the operation result
    let mark_mask = mark.marked_bits().to_u64();
    // determine for each input bit separately
    let mut left_result = 0;
    for i in 0..output_width {
        for our in 0..(1 << input_width) {
            if our & (1 << i) != 0 {
                continue;
            }
            let with_zero = RConcreteBitvector::new(our, RBound::new(input_width));
            let with_one = if input_width > 0 {
                RConcreteBitvector::new(our | (1 << i), RBound::new(input_width))
            } else {
                with_zero
            };
            if !left_abstr.contains_concrete(&with_zero) || !left_abstr.contains_concrete(&with_one)
            {
                continue;
            }
            for other in 0..(1 << input_width) {
                if !right_abstr
                    .contains_concrete(&RConcreteBitvector::new(other, RBound::new(input_width)))
                {
                    continue;
                }
                let other = RConcreteBitvector::new(other, RBound::new(input_width));
                if concr_func(with_zero, other).to_u64() & mark_mask
                    != concr_func(with_one, other).to_u64() & mark_mask
                {
                    left_result |= 1 << i;
                }
            }
        }
    }
    RMarkBitvector::new_from_flag(RConcreteBitvector::new(
        left_result,
        RBound::new(output_width),
    ))
}

fn exec_left_check(
    mark_func: impl Fn((RThreeValuedBitvector, RThreeValuedBitvector), RMarkBitvector) -> RMarkBitvector,
    concr_func: impl Fn(RConcreteBitvector, RConcreteBitvector) -> RConcreteBitvector,
    input_width: u32,
    output_width: u32,
    want_exact: bool,
) {
    // a mark bit is necessary if changing the input bit can impact the output
    // test this for all concretizations of the input

    for later in RConcreteBitvector::all_with_bound_iter(RBound::new(output_width)) {
        let later = RMarkBitvector::new_from_flag(later);

        for a_abstr in RThreeValuedBitvector::all_with_bound_iter(RBound::new(input_width)) {
            for b_abstr in RThreeValuedBitvector::all_with_bound_iter(RBound::new(input_width)) {
                let exact_earlier = exact_left_mark(
                    (a_abstr, b_abstr),
                    later,
                    &concr_func,
                    input_width,
                    output_width,
                );
                let our_earlier = mark_func((a_abstr, b_abstr), later);

                eval_mark(
                    want_exact,
                    exact_earlier,
                    our_earlier,
                    later.importance() > 0,
                );
            }
        }
    }
}

pub(super) fn exec_bi_check(
    mark_func: &impl Fn(
        (RThreeValuedBitvector, RThreeValuedBitvector),
        RMarkBitvector,
    ) -> (RMarkBitvector, RMarkBitvector),
    concr_func: &impl Fn(RConcreteBitvector, RConcreteBitvector) -> RConcreteBitvector,
    width: u32,
    want_exact: bool,
) {
    // exec for left
    let left_mark_func = |abstr, earlier| mark_func(abstr, earlier).0;
    let left_concr_func = concr_func;
    exec_left_check(left_mark_func, left_concr_func, width, width, want_exact);
    // flip for right
    let right_mark_func = |(a, b), earlier| mark_func((b, a), earlier).1;
    let right_concr_func = |a, b| concr_func(b, a);
    exec_left_check(right_mark_func, right_concr_func, width, width, want_exact);
}

pub(super) fn exec_cmp_check(
    mark_func: &impl Fn(
        (RThreeValuedBitvector, RThreeValuedBitvector),
        crate::refin::Boolean,
    ) -> (RMarkBitvector, RMarkBitvector),
    concr_func: &impl Fn(RConcreteBitvector, RConcreteBitvector) -> crate::concr::Boolean,
    input_width: u32,
    want_exact: bool,
) {
    // exec for left
    let left_mark_func =
        |abstr, earlier: RMarkBitvector| mark_func(abstr, earlier.to_condition()).0;
    let left_concr_func = |a, b| concrete_boolean_to_bitvector(concr_func(a, b));
    exec_left_check(left_mark_func, left_concr_func, input_width, 1, want_exact);
    // flip for right
    let right_mark_func =
        |(a, b), earlier: RMarkBitvector| mark_func((b, a), earlier.to_condition()).1;
    let right_concr_func = |a, b| concrete_boolean_to_bitvector(concr_func(b, a));
    exec_left_check(
        right_mark_func,
        right_concr_func,
        input_width,
        1,
        want_exact,
    );
}

pub(super) fn exec_divrem_check(
    mark_func: &impl Fn(
        (RThreeValuedBitvector, RThreeValuedBitvector),
        (RMarkBitvector, RMarkBitvector),
    ) -> (RMarkBitvector, RMarkBitvector),
    concr_func: &impl Fn(
        RConcreteBitvector,
        RConcreteBitvector,
    ) -> (RConcreteBitvector, RConcreteBitvector),
    width: u32,
    want_exact: bool,
) {
    // test with no panic first
    // exec for left
    let left_mark_func =
        |abstr, earlier| mark_func(abstr, (earlier, RMarkBitvector::new_unmarked(32))).0;
    let left_concr_func = |a, b| concr_func(a, b).0;
    exec_left_check(left_mark_func, left_concr_func, width, width, want_exact);
    // flip for right
    let right_mark_func =
        |(a, b), earlier| mark_func((b, a), (earlier, RMarkBitvector::new_unmarked(32))).1;
    let right_concr_func = |a, b| concr_func(b, a).0;
    exec_left_check(right_mark_func, right_concr_func, width, width, want_exact);

    // test that panic propagates
    for a_abstr in RThreeValuedBitvector::all_with_bound_iter(RBound::new(width)) {
        for b_abstr in RThreeValuedBitvector::all_with_bound_iter(RBound::new(width)) {
            let later_mark_result = RMarkBitvector::new_unmarked(width);
            let later_mark_panic = RMarkBitvector::new_marked_unimportant(32);
            let (marked_a, marked_b) =
                mark_func((a_abstr, b_abstr), (later_mark_result, later_mark_panic));
            if marked_a.importance() > 0 {
                panic!("Dividend should never be marked for propagating div/rem panic");
            }

            let expected_mark = RMarkBitvector::new_marked_unimportant(width).limit(&b_abstr);

            if !expected_mark.meta_eq(&marked_b) {
                panic!("Expected dividend mark for panic differs from the actual")
            }
        }
    }
}

fn concrete_boolean_to_bitvector(boolean: concr::Boolean) -> RConcreteBitvector {
    RConcreteBitvector::new(boolean.value() as u64, RBound::new(1))
}
