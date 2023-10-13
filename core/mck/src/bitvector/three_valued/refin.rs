#[cfg(test)]
mod test;

use crate::{
    backward::{self, Bitwise, Ext, HwArith, TypedCmp, TypedEq},
    bitvector::abstr,
    bitvector::concr,
    bitvector::util::compute_u64_sign_bit_mask,
    forward::{self},
    refin::Refinable,
    traits::{misc::Meta, refin::Refine},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarkBitvector<const L: u32>(concr::Bitvector<L>);

impl<const L: u32> Refinable for abstr::Bitvector<L> {
    type Refin = MarkBitvector<L>;

    fn clean_refin(&self) -> Self::Refin {
        MarkBitvector::new_unmarked()
    }
}

impl<const L: u32> MarkBitvector<L> {
    pub fn new_unmarked() -> Self {
        MarkBitvector(concr::Bitvector::new(0))
    }
    pub fn new_marked() -> Self {
        if L == 0 {
            return Self(concr::Bitvector::new(0));
        }
        let zero = concr::Bitvector::new(0);
        let one = concr::Bitvector::new(1);
        MarkBitvector(forward::HwArith::sub(zero, one))
    }
    pub fn new_from_flag(marked_flag: concr::Bitvector<L>) -> Self {
        MarkBitvector(marked_flag)
    }
    fn limit(&self, abstract_bitvec: abstr::Bitvector<L>) -> MarkBitvector<L> {
        MarkBitvector(forward::Bitwise::bitand(
            self.0,
            abstract_bitvec.get_unknown_bits(),
        ))
        //MarkBitvector(self.0)
    }
}

impl<const L: u32> Meta<abstr::Bitvector<L>> for MarkBitvector<L> {
    fn proto_first(&self) -> abstr::Bitvector<L> {
        // all known bits are 0
        let known_bits = self.0.as_unsigned();
        abstr::Bitvector::new_value_known(
            concr::Bitvector::new(0),
            concr::Bitvector::new(known_bits),
        )
    }

    fn proto_increment(&self, proto: &mut abstr::Bitvector<L>) -> bool {
        // the marked bits should be split into possibilities
        let known_bits = self.0.as_unsigned();

        if known_bits == 0 {
            // if full-unknown, stop immediately after first to avoid shl overflow
            return false;
        }

        // manual addition-style updates: only update marked positions
        // start with lowest marked position
        // if it is 0 within current, update it to 1 and end
        // if it is 1, update it to 0, temporarily forget mark and update next
        // end if we overflow

        // work with bitvector of only values, the unknowns do not change
        let mut current = proto.umin().as_unsigned();
        let mut considered_bits = known_bits;

        loop {
            let one_pos = considered_bits.trailing_zeros();
            let one_mask = 1u64 << one_pos;
            if current & one_mask == 0 {
                // if considered bit is 0 within current, update it to 1 and end
                current |= one_mask;
                let result = abstr::Bitvector::new_value_known(
                    concr::Bitvector::new(current),
                    concr::Bitvector::new(known_bits),
                );

                *proto = result;
                return true;
            }
            // if it is 1, update it to 0, temporarily do not consider it and update next
            current &= !one_mask;
            considered_bits &= !one_mask;

            // end if we overflow
            // reset possibility to allow for cycling
            if considered_bits == 0 {
                *proto = self.proto_first();
                return false;
            }
        }
    }
}

impl<const L: u32> Refine<abstr::Bitvector<L>> for MarkBitvector<L> {
    fn apply_join(&mut self, other: &Self) {
        self.0 = forward::Bitwise::bitor(self.0, other.0);
    }

    fn apply_refin(&mut self, offer: &Self) -> bool {
        // find the highest bit that is marked in offer but unmarked in ours
        let applicants = forward::Bitwise::bitand(offer.0, forward::Bitwise::not(self.0));
        let mark_mask = 1u64.checked_shl(applicants.as_unsigned().trailing_zeros());
        let Some(mark_mask) = mark_mask else {
            // no such bit found
            return false;
        };
        // apply the mark
        self.0 = forward::Bitwise::bitor(self.0, concr::Bitvector::new(mark_mask));
        true
    }

    fn force_decay(&self, target: &mut abstr::Bitvector<L>) {
        // unmarked fields become unknown
        let forced_unknown = forward::HwArith::neg(self.0);
        let zeros = forward::Bitwise::bitor(target.get_possibly_zero_flags(), forced_unknown);
        let ones = forward::Bitwise::bitor(target.get_possibly_one_flags(), forced_unknown);
        *target = abstr::Bitvector::from_zeros_ones(zeros, ones);
    }
}

impl<const L: u32> Default for MarkBitvector<L> {
    fn default() -> Self {
        Self::new_unmarked()
    }
}

impl<const L: u32> TypedEq for abstr::Bitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<1>;

    fn typed_eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        // every unknown bit may be responsible
        let extended = MarkBitvector(forward::Ext::sext(mark_later.0));
        (
            extended.limit(normal_input.0),
            extended.limit(normal_input.1),
        )
    }
}

fn default_uni_mark<const L: u32, const X: u32>(
    normal_input: (abstr::Bitvector<L>,),
    mark_later: MarkBitvector<X>,
) -> (MarkBitvector<L>,) {
    if mark_later == MarkBitvector::new_unmarked() {
        return (MarkBitvector::new_unmarked(),);
    }
    (MarkBitvector::new_marked().limit(normal_input.0),)
}

fn default_bi_mark<const L: u32, const X: u32>(
    normal_input: (abstr::Bitvector<L>, abstr::Bitvector<L>),
    mark_later: MarkBitvector<X>,
) -> (MarkBitvector<L>, MarkBitvector<L>) {
    if mark_later == MarkBitvector::new_unmarked() {
        return (MarkBitvector::new_unmarked(), MarkBitvector::new_unmarked());
    }
    (
        MarkBitvector::new_marked().limit(normal_input.0),
        MarkBitvector::new_marked().limit(normal_input.1),
    )
}

impl<const L: u32> Bitwise for abstr::Bitvector<L> {
    type Mark = MarkBitvector<L>;

    fn not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        // propagate marking of given bits with limitation
        (mark_later.limit(normal_input.0),)
    }

    fn bitand(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        (
            mark_later.limit(normal_input.0),
            mark_later.limit(normal_input.1),
        )
    }

    fn bitor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        (
            mark_later.limit(normal_input.0),
            mark_later.limit(normal_input.1),
        )
    }

    fn bitxor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        (
            mark_later.limit(normal_input.0),
            mark_later.limit(normal_input.1),
        )
    }
}

impl<const L: u32> HwArith for abstr::Bitvector<L> {
    type Mark = MarkBitvector<L>;

    fn neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        default_uni_mark(normal_input, mark_later)
    }

    fn add(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }

    fn sub(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }

    fn mul(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }

    fn udiv(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }

    fn sdiv(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }

    fn urem(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }

    fn srem(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }
}

impl<const L: u32> TypedCmp for abstr::Bitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<1>;

    fn typed_slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, mark_later)
    }

    fn typed_ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, mark_later)
    }

    fn typed_slte(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, mark_later)
    }

    fn typed_ulte(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, mark_later)
    }
}

impl<const L: u32, const X: u32> Ext<X> for abstr::Bitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<X>;

    fn uext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        // we are going in reverse
        // but unsigned extension does not transport any unknown bit
        // propagate marking of given bits with limitation
        let extended = MarkBitvector(crate::forward::Ext::uext(mark_later.0));
        (extended.limit(normal_input.0),)
    }

    fn sext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        // we are going in reverse

        // in case forward signed extension cut the bitvector or did not do anything,
        // the there was no transport of any unknown bit

        // in case forward signed extension really extended the bitvector, new high bits were added
        // as a copy of the sign bit, propagate marking from these high bits back to the sign bit

        // do unsigned extension and then treat the potential high bits specially

        let mut extended = crate::forward::Ext::<L>::uext(mark_later.0);

        if X > L {
            let back = MarkBitvector(crate::forward::Ext::<X>::uext(extended));
            if mark_later != back {
                // propagate marking to the sign bit
                extended = crate::forward::Bitwise::bitor(
                    extended,
                    concr::Bitvector::new(compute_u64_sign_bit_mask(L)),
                );
            }
        }

        let extended = MarkBitvector(extended);

        (extended.limit(normal_input.0),)
    }
}

fn shift<const L: u32>(
    normal_input: (abstr::Bitvector<L>, abstr::Bitvector<L>),
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
        if amount_input.can_contain(machine_i) {
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

impl<const L: u32> backward::HwShift for abstr::Bitvector<L> {
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
