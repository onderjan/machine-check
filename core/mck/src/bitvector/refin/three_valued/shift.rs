use crate::{
    abstr::BitvectorDomain,
    backward,
    bitvector::{
        abstr::three_valued::RThreeValuedBitvector,
        refin::three_valued::{RBitvectorMark, RMarkBitvector},
        util::compute_u64_sign_bit_mask,
        RBound,
    },
    concr::RConcreteBitvector,
    forward,
    misc::BitvectorBound,
    refin::RefinementDomain,
};

impl backward::HwShift for RThreeValuedBitvector {
    type Mark = RMarkBitvector;

    fn logic_shl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        assert_eq!(normal_input.0.bound(), normal_input.1.bound());
        let width = normal_input.0.bound().width();

        let Some(mark_later) = mark_later.inner else {
            return (
                RMarkBitvector::new_unmarked(width),
                RMarkBitvector::new_unmarked(width),
            );
        };

        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use srl
        runtime_shift(normal_input, mark_later, |a, b| {
            forward::HwShift::logic_shr(a, b)
        })
    }

    fn logic_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        assert_eq!(normal_input.0.bound(), normal_input.1.bound());
        let width = normal_input.0.bound().width();

        let Some(mark_later) = mark_later.inner else {
            return (
                RMarkBitvector::new_unmarked(width),
                RMarkBitvector::new_unmarked(width),
            );
        };

        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use sll
        runtime_shift(normal_input, mark_later, |a, b| {
            forward::HwShift::logic_shl(a, b)
        })
    }

    fn arith_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        assert_eq!(
            normal_input.0.bound().width(),
            normal_input.1.bound().width()
        );
        let width = normal_input.0.bound().width();

        let Some(mark_later) = mark_later.inner else {
            return (
                RMarkBitvector::new_unmarked(width),
                RMarkBitvector::new_unmarked(width),
            );
        };

        if width == 0 {
            // avoid problems with zero-width bitvectors+
            let importance = mark_later.importance;
            return (
                RMarkBitvector::new_marked(importance, width),
                RMarkBitvector::new_marked(importance, width),
            );
        }

        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use sll and then manually set the sign bit if some left-shifted-out bit was marked
        runtime_shift(normal_input, mark_later, |a, b| {
            let mut result = forward::HwShift::logic_shl(a, b);
            let back = forward::HwShift::logic_shr(result, b);
            if a != back {
                // mark the sign bit of result
                result = forward::Bitwise::bit_or(
                    result,
                    RConcreteBitvector::new(compute_u64_sign_bit_mask(width), RBound::new(width)),
                );
            }
            result
        })
    }
}

fn runtime_shift(
    normal_input: (RThreeValuedBitvector, RThreeValuedBitvector),
    mark_later: RBitvectorMark,
    shift_fn: impl Fn(RConcreteBitvector, RConcreteBitvector) -> RConcreteBitvector,
) -> (RMarkBitvector, RMarkBitvector) {
    let width = normal_input.0.bound().width();
    if width == 0 {
        // avoid problems with zero-width bitvectors
        return (
            RMarkBitvector::new_marked(mark_later.importance, width),
            RMarkBitvector::new_marked(mark_later.importance, width),
        );
    }

    // for now, only do detailed marking of value to be shifted, not the shift amount
    let amount_input = normal_input.1;

    // the shift amount is also three-valued, which poses problems
    // if the shift amount is L or more, no bits are retained
    // so consider only lesser amounts one by one

    let min_shift = amount_input.umin().to_u64().min((width - 1) as u64);
    let max_shift = amount_input.umax().to_u64().max((width - 1) as u64);
    // join the shifted marks iteratively
    let mut shifted_mark_earlier = RMarkBitvector::new_unmarked(width);
    for i in min_shift..=max_shift {
        let machine_i = RConcreteBitvector::new(i, RBound::new(width));
        if amount_input.contains_concrete(&machine_i) {
            // shift the mark
            let shifted_mark = shift_fn(mark_later.mark, machine_i);
            shifted_mark_earlier.apply_join(&RMarkBitvector {
                inner: Some(RBitvectorMark {
                    mark: shifted_mark,
                    importance: mark_later.importance,
                }),
                width,
            });
        }
    }
    (
        shifted_mark_earlier.limit(&normal_input.0),
        RMarkBitvector::new_marked(mark_later.importance, width).limit(&normal_input.1),
    )
}
