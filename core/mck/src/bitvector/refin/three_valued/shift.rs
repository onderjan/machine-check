use crate::{
    backward,
    bitvector::{abstr::ThreeValuedBitvector, util::compute_u64_sign_bit_mask},
    concr::ConcreteBitvector,
    forward,
    traits::refin::Refine,
};

use super::{BitvectorMark, MarkBitvector};

impl<const W: u32> backward::HwShift for ThreeValuedBitvector<W> {
    type Mark = MarkBitvector<W>;

    fn logic_shl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        let Some(mark_later) = mark_later.0 else {
            return (MarkBitvector::new_unmarked(), MarkBitvector::new_unmarked());
        };

        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use srl
        shift(normal_input, mark_later, |a, b| {
            forward::HwShift::logic_shr(a, b)
        })
    }

    fn logic_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        let Some(mark_later) = mark_later.0 else {
            return (MarkBitvector::new_unmarked(), MarkBitvector::new_unmarked());
        };

        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use sll
        shift(normal_input, mark_later, |a, b| {
            forward::HwShift::logic_shl(a, b)
        })
    }

    fn arith_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        let Some(mark_later) = mark_later.0 else {
            return (MarkBitvector::new_unmarked(), MarkBitvector::new_unmarked());
        };

        if W == 0 {
            // avoid problems with zero-width bitvectors+
            let importance = mark_later.importance;
            return (
                MarkBitvector::new_marked(importance),
                MarkBitvector::new_marked(importance),
            );
        }

        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use sll and then manually set the sign bit if some left-shifted-out bit was marked
        shift(normal_input, mark_later, |a, b| {
            let mut result = forward::HwShift::logic_shl(a, b);
            let back = forward::HwShift::logic_shr(result, b);
            if a != back {
                // mark the sign bit of result
                result = forward::Bitwise::bit_or(
                    result,
                    ConcreteBitvector::new(compute_u64_sign_bit_mask(W)),
                );
            }
            result
        })
    }
}

fn shift<const W: u32>(
    normal_input: (ThreeValuedBitvector<W>, ThreeValuedBitvector<W>),
    mark_later: BitvectorMark<W>,
    shift_fn: fn(ConcreteBitvector<W>, ConcreteBitvector<W>) -> ConcreteBitvector<W>,
) -> (MarkBitvector<W>, MarkBitvector<W>) {
    if W == 0 {
        // avoid problems with zero-width bitvectors
        return (
            MarkBitvector::new_marked(mark_later.importance),
            MarkBitvector::new_marked(mark_later.importance),
        );
    }

    // for now, only do detailed marking of value to be shifted, not the shift amount
    let amount_input = normal_input.1;

    // the shift amount is also three-valued, which poses problems
    // if the shift amount is L or more, no bits are retained
    // so consider only lesser amounts one by one

    let min_shift = amount_input.umin().to_u64().min((W - 1) as u64);
    let max_shift = amount_input.umax().to_u64().max((W - 1) as u64);
    // join the shifted marks iteratively
    let mut shifted_mark_earlier = MarkBitvector::new_unmarked();
    for i in min_shift..=max_shift {
        let machine_i = ConcreteBitvector::new(i);
        if amount_input.contains_concr(&machine_i) {
            // shift the mark
            let shifted_mark = shift_fn(mark_later.mark, machine_i);
            shifted_mark_earlier.apply_join(&MarkBitvector(Some(BitvectorMark {
                mark: shifted_mark,
                importance: mark_later.importance,
            })));
        }
    }
    (
        shifted_mark_earlier.limit(normal_input.0),
        MarkBitvector::new_marked(mark_later.importance).limit(normal_input.1),
    )
}
