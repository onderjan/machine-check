use crate::{
    backward,
    bitvector::{
        concr, three_valued::abstr::ThreeValuedBitvector, util::compute_u64_sign_bit_mask,
    },
    forward,
    traits::refin::Refine,
};

use super::MarkBitvector;

impl<const L: u32> backward::HwShift for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn logic_shl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use srl
        shift(normal_input, mark_later, |a, b| {
            forward::HwShift::logic_shr(a, b)
        })
    }

    fn logic_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use sll
        shift(normal_input, mark_later, |a, b| {
            forward::HwShift::logic_shl(a, b)
        })
    }

    fn arith_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        if L == 0 {
            // avoid problems with zero-width bitvectors
            return (MarkBitvector::new_marked(), MarkBitvector::new_marked());
        }

        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use sll and then manually set the sign bit if some left-shifted-out bit was marked
        shift(normal_input, mark_later, |a, b| {
            let mut result = forward::HwShift::logic_shl(a, b);
            let back = forward::HwShift::logic_shr(result, b);
            if a != back {
                // mark the sign bit of result
                result = forward::Bitwise::bitor(
                    result,
                    concr::Bitvector::new(compute_u64_sign_bit_mask(L)),
                );
            }
            result
        })
    }
}

fn shift<const L: u32>(
    normal_input: (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
    mark_later: MarkBitvector<L>,
    shift_fn: fn(concr::Bitvector<L>, concr::Bitvector<L>) -> concr::Bitvector<L>,
) -> (MarkBitvector<L>, MarkBitvector<L>) {
    if mark_later == MarkBitvector::new_unmarked() {
        // avoid spurious marking of shift amount
        return (MarkBitvector::new_unmarked(), MarkBitvector::new_unmarked());
    }
    if L == 0 {
        // avoid problems with zero-width bitvectors
        return (MarkBitvector::new_marked(), MarkBitvector::new_marked());
    }

    // for now, only do detailed marking of value to be shifted, not the shift amount
    let amount_input = normal_input.1;

    // the shift amount is also three-valued, which poses problems
    // if the shift amount is L or more, no bits are retained
    // so consider only lesser amounts one by one

    let min_shift = amount_input.umin().as_unsigned().min((L - 1) as u64);
    let max_shift = amount_input.umax().as_unsigned().max((L - 1) as u64);
    // join the shifted marks iteratively
    let mut shifted_mark_earlier = MarkBitvector::new_unmarked();
    for i in min_shift..=max_shift {
        let machine_i = concr::Bitvector::new(i);
        if amount_input.contains_concr(&machine_i) {
            // shift the mark
            let shifted_mark = shift_fn(mark_later.0, machine_i);
            shifted_mark_earlier.apply_join(&MarkBitvector(shifted_mark));
        }
    }
    (
        shifted_mark_earlier.limit(normal_input.0),
        MarkBitvector::new_marked().limit(normal_input.1),
    )
}
