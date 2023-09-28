use std::num::Wrapping;

use crate::{
    mark::{
        Add, BitAnd, BitOr, BitXor, Join, MachineExt, MachineShift, Markable, Mul, Neg, Not, Sub,
        TypedCmp, TypedEq,
    },
    MachineBitvector, Possibility, ThreeValuedBitvector,
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
        let zero = MachineBitvector::new(0);
        let one = MachineBitvector::new(1);
        MarkBitvector(zero - one)
    }
    pub fn new_from_flag(marked_flag: MachineBitvector<L>) -> Self {
        MarkBitvector(marked_flag)
    }
    fn limit(&self, abstract_bitvec: ThreeValuedBitvector<L>) -> MarkBitvector<L> {
        MarkBitvector(self.0 & abstract_bitvec.get_unknown_bits())
    }
}

impl<const L: u32> Possibility for MarkBitvector<L> {
    type Possibility = ThreeValuedBitvector<L>;

    fn first_possibility(&self) -> ThreeValuedBitvector<L> {
        // all known bits are 0
        let known_bits = self.0.concrete_value();
        ThreeValuedBitvector::new_value_known(Wrapping(0), known_bits)
    }

    fn increment_possibility(&self, possibility: &mut ThreeValuedBitvector<L>) -> bool {
        // the marked bits should be split into possibilities
        let known_bits = self.0.concrete_value();

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
        let mut current = possibility.umin();
        let mut considered_bits = known_bits;

        loop {
            let one_pos = considered_bits.0.trailing_zeros();
            let one_mask = Wrapping(1u64 << one_pos);
            if current & one_mask == Wrapping(0) {
                // if considered bit is 0 within current, update it to 1 and end
                current |= one_mask;
                let result = ThreeValuedBitvector::new_value_known(current, known_bits);

                *possibility = result;
                return true;
            }
            // if it is 1, update it to 0, temporarily do not consider it and update next
            current &= !one_mask;
            considered_bits &= !one_mask;

            // end if we overflow
            // reset possibility to allow for cycling
            if considered_bits == Wrapping(0) {
                *possibility = self.first_possibility();
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

impl<const L: u32> Neg for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        // TODO: improve, just mark everything for now

        (Self::Mark::new_marked().limit(normal_input.0),)
    }
}

impl<const L: u32> Add for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn add(normal_input: (Self, Self), _mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // TODO: improve, just mark everything for now

        (
            Self::Mark::new_marked().limit(normal_input.0),
            Self::Mark::new_marked().limit(normal_input.1),
        )
    }
}
impl<const L: u32> Sub for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn sub(normal_input: (Self, Self), _mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // TODO: improve, just mark everything for now

        (
            Self::Mark::new_marked().limit(normal_input.0),
            Self::Mark::new_marked().limit(normal_input.1),
        )
    }
}

impl<const L: u32> Mul for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn mul(normal_input: (Self, Self), _mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // TODO: improve, just mark everything for now
        (
            Self::Mark::new_marked().limit(normal_input.0),
            Self::Mark::new_marked().limit(normal_input.1),
        )
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

impl<const L: u32> TypedCmp for ThreeValuedBitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<1>;

    fn typed_sgt(
        _normal_input: (Self, Self),
        _mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_ugt(
        _normal_input: (Self, Self),
        _mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_sgte(
        _normal_input: (Self, Self),
        _mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_ugte(
        _normal_input: (Self, Self),
        _mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_slt(
        _normal_input: (Self, Self),
        _mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_ult(
        _normal_input: (Self, Self),
        _mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_slte(
        _normal_input: (Self, Self),
        _mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_ulte(
        _normal_input: (Self, Self),
        _mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }
}

impl<const L: u32, const X: u32> MachineExt<X> for ThreeValuedBitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<X>;

    fn uext(_normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        // unsigned extension does not add any bit
        // propagate marking of given bits with limitation
        let extended = MarkBitvector(crate::MachineExt::uext(mark_later.0));
        //(extended.limit(normal_input.0),)
        (extended,)
    }

    fn sext(_normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        // signed extension copies high bit
        // copy it in marking with signed extension
        let extended = MarkBitvector(crate::MachineExt::sext(mark_later.0));
        //(extended.limit(normal_input.0),)
        (extended,)
    }
}

impl<const L: u32> MachineShift for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn sll(_normal_input: (Self, Self), _mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn srl(_normal_input: (Self, Self), _mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn sra(_normal_input: (Self, Self), _mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        todo!()
    }
}
