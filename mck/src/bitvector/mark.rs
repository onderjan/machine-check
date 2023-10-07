#[cfg(test)]
mod test;

use std::num::Wrapping;

use crate::{
    mark::{
        Add, BitAnd, BitOr, BitXor, Join, MachineDiv, MachineExt, MachineShift, MarkSingle,
        Markable, Mul, Neg, Not, Sub, TypedCmp, TypedEq,
    },
    util::compute_sign_bit_mask,
    Fabricator, MachineBitvector, ThreeValuedBitvector,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarkBitvector<const L: u32>(MachineBitvector<L>);

impl<const L: u32> Markable for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn create_clean_mark(&self) -> Self::Mark {
        MarkBitvector::new_unmarked()
    }
}

impl<const L: u32> MarkBitvector<L> {
    pub fn new_unmarked() -> Self {
        MarkBitvector(MachineBitvector::new(0))
    }
    pub fn new_marked() -> Self {
        if L == 0 {
            return Self(MachineBitvector::new(0));
        }
        let zero = MachineBitvector::new(0);
        let one = MachineBitvector::new(1);
        MarkBitvector(zero - one)
    }
    pub fn new_from_flag(marked_flag: MachineBitvector<L>) -> Self {
        MarkBitvector(marked_flag)
    }
    fn limit(&self, abstract_bitvec: ThreeValuedBitvector<L>) -> MarkBitvector<L> {
        MarkBitvector(self.0 & abstract_bitvec.get_unknown_bits())
        //MarkBitvector(self.0)
    }
}

impl<const L: u32> Fabricator for MarkBitvector<L> {
    type Fabricated = ThreeValuedBitvector<L>;

    fn fabricate_first(&self) -> ThreeValuedBitvector<L> {
        // all known bits are 0
        let known_bits = self.0.as_unsigned();
        ThreeValuedBitvector::new_value_known(Wrapping(0), known_bits)
    }

    fn increment_fabricated(&self, fabricated: &mut ThreeValuedBitvector<L>) -> bool {
        // the marked bits should be split into possibilities
        let known_bits = self.0.as_unsigned();

        if known_bits == Wrapping(0) {
            // if full-unknown, stop immediately after first to avoid shl overflow
            return false;
        }

        // manual addition-style updates: only update marked positions
        // start with lowest marked position
        // if it is 0 within current, update it to 1 and end
        // if it is 1, update it to 0, temporarily forget mark and update next
        // end if we overflow

        // work with bitvector of only values, the unknowns do not change
        let mut current = fabricated.umin();
        let mut considered_bits = known_bits;

        loop {
            let one_pos = considered_bits.0.trailing_zeros();
            let one_mask = Wrapping(1u64 << one_pos);
            if current & one_mask == Wrapping(0) {
                // if considered bit is 0 within current, update it to 1 and end
                current |= one_mask;
                let result = ThreeValuedBitvector::new_value_known(current, known_bits);

                *fabricated = result;
                return true;
            }
            // if it is 1, update it to 0, temporarily do not consider it and update next
            current &= !one_mask;
            considered_bits &= !one_mask;

            // end if we overflow
            // reset possibility to allow for cycling
            if considered_bits == Wrapping(0) {
                *fabricated = self.fabricate_first();
                return false;
            }
        }
    }
}

impl<const L: u32> Join for MarkBitvector<L> {
    fn apply_join(&mut self, other: Self) {
        self.0 = self.0 | other.0;
    }
}

impl<const L: u32> MarkSingle for MarkBitvector<L> {
    fn apply_single_mark(&mut self, offer: Self) -> bool {
        // find the highest bit that is marked in offer but unmarked in ours
        let applicants = offer.0 & !self.0;
        let mark_mask = 1u64.checked_shl(applicants.as_unsigned().0.trailing_zeros());
        let Some(mark_mask) = mark_mask else {
            // no such bit found
            return false;
        };
        // apply the mark
        self.0 = self.0 | MachineBitvector::new(mark_mask);
        true
    }
}

impl<const L: u32> Default for MarkBitvector<L> {
    fn default() -> Self {
        Self::new_unmarked()
    }
}

impl<const L: u32> TypedEq for ThreeValuedBitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<1>;

    fn typed_eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        // every unknown bit may be responsible
        let extended = MarkBitvector(crate::MachineExt::sext(mark_later.0));
        (
            extended.limit(normal_input.0),
            extended.limit(normal_input.1),
        )
    }
}

fn default_uni_mark<const L: u32, const X: u32>(
    normal_input: (ThreeValuedBitvector<L>,),
    mark_later: MarkBitvector<X>,
) -> (MarkBitvector<L>,) {
    if mark_later == MarkBitvector::new_unmarked() {
        return (MarkBitvector::new_unmarked(),);
    }
    (MarkBitvector::new_marked().limit(normal_input.0),)
}

fn default_bi_mark<const L: u32, const X: u32>(
    normal_input: (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
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

impl<const L: u32> Neg for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        default_uni_mark(normal_input, mark_later)
    }
}

impl<const L: u32> Add for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn add(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }
}
impl<const L: u32> Sub for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn sub(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }
}

impl<const L: u32> Mul for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn mul(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }
}

impl<const L: u32> Not for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        // propagate marking of given bits with limitation
        (mark_later.limit(normal_input.0),)
    }
}

impl<const L: u32> BitAnd for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn bitand(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        (
            mark_later.limit(normal_input.0),
            mark_later.limit(normal_input.1),
        )
    }
}
impl<const L: u32> BitOr for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn bitor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        (
            mark_later.limit(normal_input.0),
            mark_later.limit(normal_input.1),
        )
    }
}
impl<const L: u32> BitXor for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn bitxor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        (
            mark_later.limit(normal_input.0),
            mark_later.limit(normal_input.1),
        )
    }
}

impl<const L: u32> MachineDiv for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn sdiv(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }

    fn udiv(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }

    fn smod(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }

    fn srem(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }

    fn urem(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }
}

impl<const L: u32> TypedCmp for ThreeValuedBitvector<L> {
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

impl<const L: u32, const X: u32> MachineExt<X> for ThreeValuedBitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<X>;

    fn uext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        // we are going in reverse
        // but unsigned extension does not transport any unknown bit
        // propagate marking of given bits with limitation
        let extended = MarkBitvector(crate::MachineExt::uext(mark_later.0));
        (extended.limit(normal_input.0),)
    }

    fn sext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        // we are going in reverse

        // in case forward signed extension cut the bitvector or did not do anything,
        // the there was no transport of any unknown bit

        // in case forward signed extension really extended the bitvector, new high bits were added
        // as a copy of the sign bit, propagate marking from these high bits back to the sign bit

        // do unsigned extension and then treat the potential high bits specially

        let mut extended = crate::MachineExt::<L>::uext(mark_later.0);

        if X > L {
            let back = MarkBitvector(crate::MachineExt::<X>::uext(extended));
            if mark_later != back {
                // propagate marking to the sign bit
                extended = extended | MachineBitvector::new(compute_sign_bit_mask(L).0);
            }
        }

        let extended = MarkBitvector(extended);

        (extended.limit(normal_input.0),)
    }
}

fn shift<const L: u32>(
    normal_input: (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
    mark_later: MarkBitvector<L>,
    shift_fn: fn(MachineBitvector<L>, MachineBitvector<L>) -> MachineBitvector<L>,
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

    let min_shift = amount_input.umin().0.min((L - 1) as u64);
    let max_shift = amount_input.umax().0.max((L - 1) as u64);
    // join the shifted marks iteratively
    let mut shifted_mark_earlier = MarkBitvector::new_unmarked();
    for i in min_shift..=max_shift {
        if amount_input.can_contain(Wrapping(i)) {
            // shift the mark
            let machine_i = MachineBitvector::new(i);
            let shifted_mark = shift_fn(mark_later.0, machine_i);
            shifted_mark_earlier.apply_join(MarkBitvector(shifted_mark));
        }
    }
    (
        shifted_mark_earlier.limit(normal_input.0),
        MarkBitvector::new_marked().limit(normal_input.1),
    )
}

impl<const L: u32> MachineShift for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn sll(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use srl
        shift(normal_input, mark_later, |a, b| {
            crate::MachineShift::srl(a, b)
        })
    }

    fn srl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use sll
        shift(normal_input, mark_later, |a, b| {
            crate::MachineShift::sll(a, b)
        })
    }

    fn sra(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        if L == 0 {
            // avoid problems with zero-width bitvectors
            return (MarkBitvector::new_marked(), MarkBitvector::new_marked());
        }

        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use sll and then manually set the sign bit if some left-shifted-out bit was marked
        shift(normal_input, mark_later, |a, b| {
            let mut result = crate::MachineShift::sll(a, b);
            let back = crate::MachineShift::srl(result, b);
            if a != back {
                // mark the sign bit of result
                result = result | MachineBitvector::new(compute_sign_bit_mask(L).0);
            }
            result
        })
    }
}
