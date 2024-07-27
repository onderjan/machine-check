use crate::bitvector::{
    concrete::ConcreteBitvector,
    three_valued::{abstr::ThreeValuedBitvector, refin::MarkBitvector},
};

macro_rules! uni_op_test {
    ($ty:tt, $op:tt, $exact:tt) => {

        seq_macro::seq!(L in 0..=6 {

        #[test]
        pub fn $op~L() {
            let mark_func = |a,
                                a_mark: MarkBitvector<L>|
                -> MarkBitvector<L> { $crate::backward::$ty::$op((a,), a_mark).0 };
            let concr_func = $crate::forward::$ty::$op;
            $crate::bitvector::three_valued::refin::tests::op::exec_uni_check(mark_func, concr_func, $exact);
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
                                a_mark: MarkBitvector<X>|
                -> MarkBitvector<L> { $crate::backward::$ty::$op((a,), a_mark).0 };
                let concr_func = $crate::forward::$ty::$op;
                $crate::bitvector::three_valued::refin::tests::op::exec_uni_check(mark_func, concr_func, $exact);
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
            let mark_func = |inputs: ($crate::bitvector::three_valued::abstr::ThreeValuedBitvector<L>,
                $crate::bitvector::three_valued::abstr::ThreeValuedBitvector<L>),
                                mark| {
                                    ::std::convert::Into::into(crate::backward::$ty::$op(inputs, ::std::convert::Into::into(mark)))
            };
            let concr_func = |a: $crate::bitvector::concrete::ConcreteBitvector<L>, b:$crate::bitvector::concrete::ConcreteBitvector<L>| ::std::convert::Into::into($crate::forward::$ty::$op(a,b));
            $crate::bitvector::three_valued::refin::tests::op::exec_bi_check(&mark_func, &concr_func, $exact);
        }
    });
    };
}

fn exact_uni_mark<const L: u32, const X: u32>(
    a_abstr: ThreeValuedBitvector<L>,
    a_mark: MarkBitvector<X>,
    concr_func: fn(ConcreteBitvector<L>) -> ConcreteBitvector<X>,
) -> MarkBitvector<L> {
    // the result marks exactly those bits of input which, if changed in operation input,
    // can change bits masked by mark_a in the operation result
    let mark_mask = a_mark.marked_bits().as_unsigned();
    // determine for each input bit separately
    let mut result = 0;
    for i in 0..L {
        for a in ConcreteBitvector::<L>::all_with_length_iter() {
            if a.as_unsigned() & (1 << i) != 0 {
                continue;
            }
            let with_zero = a;
            let with_one = ConcreteBitvector::new(a.as_unsigned() | (1 << i));
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
    MarkBitvector::new_from_flag(ConcreteBitvector::new(result))
}

fn eval_mark<const L: u32>(
    want_exact: bool,
    exact_earlier: MarkBitvector<L>,
    our_earlier: MarkBitvector<L>,
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
        let exact = exact_earlier.marked_bits().as_unsigned();
        let our = our_earlier.marked_bits().as_unsigned();
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

pub(super) fn exec_uni_check<const L: u32, const X: u32>(
    mark_func: fn(ThreeValuedBitvector<L>, MarkBitvector<X>) -> MarkBitvector<L>,
    concr_func: fn(ConcreteBitvector<L>) -> ConcreteBitvector<X>,
    want_exact: bool,
) {
    // a mark bit is necessary if changing the input bit can impact the output
    // test this for all concretizations of the input

    for a_later in ConcreteBitvector::all_with_length_iter() {
        let a_later = MarkBitvector::new_from_flag(a_later);

        for a_abstr in ThreeValuedBitvector::all_with_length_iter() {
            let exact_earlier = exact_uni_mark(a_abstr, a_later, concr_func);
            let our_earlier = mark_func(a_abstr, a_later);
            eval_mark(want_exact, exact_earlier, our_earlier, a_later.is_marked());
        }
    }
}

fn exact_left_mark<const L: u32, const X: u32>(
    abstr: (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
    mark: MarkBitvector<X>,
    concr_func: impl Fn(ConcreteBitvector<L>, ConcreteBitvector<L>) -> ConcreteBitvector<X>,
) -> MarkBitvector<L> {
    let left_abstr = abstr.0;
    let right_abstr = abstr.1;
    // the result marks exactly those bits of input which, if changed in operation input,
    // can change bits masked by mark_a in the operation result
    let mark_mask = mark.marked_bits().as_unsigned();
    // determine for each input bit separately
    let mut left_result = 0;
    for i in 0..L {
        for our in 0..(1 << L) {
            if our & (1 << i) != 0 {
                continue;
            }
            let with_zero = ConcreteBitvector::new(our);
            let with_one = ConcreteBitvector::new(our | (1 << i));
            if !left_abstr.contains_concr(&with_zero) || !left_abstr.contains_concr(&with_one) {
                continue;
            }
            for other in 0..(1 << L) {
                if !right_abstr.contains_concr(&ConcreteBitvector::new(other)) {
                    continue;
                }
                let other = ConcreteBitvector::new(other);
                if concr_func(with_zero, other).as_unsigned() & mark_mask
                    != concr_func(with_one, other).as_unsigned() & mark_mask
                {
                    left_result |= 1 << i;
                }
            }
        }
    }
    MarkBitvector::new_from_flag(ConcreteBitvector::new(left_result))
}

fn exec_left_check<const L: u32, const X: u32>(
    mark_func: impl Fn(
        (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
        MarkBitvector<X>,
    ) -> MarkBitvector<L>,
    concr_func: impl Fn(ConcreteBitvector<L>, ConcreteBitvector<L>) -> ConcreteBitvector<X>,
    want_exact: bool,
) {
    // a mark bit is necessary if changing the input bit can impact the output
    // test this for all concretizations of the input

    for a_later in ConcreteBitvector::all_with_length_iter() {
        let a_later = MarkBitvector::new_from_flag(a_later);

        for a_abstr in ThreeValuedBitvector::<L>::all_with_length_iter() {
            for b_abstr in ThreeValuedBitvector::<L>::all_with_length_iter() {
                let exact_earlier = exact_left_mark((a_abstr, b_abstr), a_later, &concr_func);
                let our_earlier = mark_func((a_abstr, b_abstr), a_later);

                eval_mark(want_exact, exact_earlier, our_earlier, a_later.is_marked());
            }
        }
    }
}

pub(super) fn exec_bi_check<const L: u32, const X: u32>(
    mark_func: &impl Fn(
        (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
        MarkBitvector<X>,
    ) -> (MarkBitvector<L>, MarkBitvector<L>),
    concr_func: &impl Fn(ConcreteBitvector<L>, ConcreteBitvector<L>) -> ConcreteBitvector<X>,
    want_exact: bool,
) {
    // exec for left
    let left_mark_func = |abstr, earlier| mark_func(abstr, earlier).0;
    let left_concr_func = concr_func;
    exec_left_check(left_mark_func, left_concr_func, want_exact);
    // flip for right
    let right_mark_func = |(a, b), earlier| mark_func((b, a), earlier).1;
    let right_concr_func = |a, b| concr_func(b, a);
    exec_left_check(right_mark_func, right_concr_func, want_exact);
}
