use std::num::Wrapping;

use crate::{
    mark::{
        self, Add, BitAnd, BitOr, BitXor, MachineExt, MachineShift, Mul, Neg, Not, Sub, TypedCmp,
        TypedEq,
    },
    MachineBitvector, ThreeValuedBitvector,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarkBitvector<const L: u32>(MachineBitvector<L>);

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

    pub fn join(self, rhs: Self) -> Self {
        MarkBitvector(self.0 | rhs.0)
    }
}

pub struct PossibilityIter<const L: u32> {
    mark: MarkBitvector<L>,
    current: Option<Wrapping<u64>>,
}

impl<const L: u32> Iterator for PossibilityIter<L> {
    type Item = ThreeValuedBitvector<L>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current = match self.current {
            Some(current) => current,
            None => return None,
        };

        // manual addition-style updates: only update marked positions
        // start with lowest marked position
        // if it is 0 within current, update it to 1 and end
        // if it is 1, update it to 0, temporarily forget mark and update next
        // end iterator if we overflow

        let known_bits = self.mark.0.concrete_value();

        if known_bits == Wrapping(0) {
            self.current = None;
            return Some(ThreeValuedBitvector::new_unknown());
        }

        let result = ThreeValuedBitvector::new_value_known(current, known_bits);

        let mut considered_bits = known_bits;

        loop {
            let one_pos = considered_bits.0.trailing_zeros();
            let one_mask = Wrapping(1u64 << one_pos);
            if current & one_mask == Wrapping(0) {
                // if it is 0 within current, update it to 1 and end
                self.current = Some(current | one_mask);
                return Some(result);
            }
            // if it is 1, update it to 0, temporarily do not consider it and update next
            current = current & !one_mask;
            considered_bits = considered_bits & !one_mask;

            // end iterator if we overflow
            if considered_bits == Wrapping(0) {
                self.current = None;
                return Some(result);
            }
        }
    }
}

impl<const L: u32> MarkBitvector<L> {
    pub fn possibility_iter(&self) -> impl Iterator<Item = ThreeValuedBitvector<L>> {
        PossibilityIter {
            mark: *self,
            current: Some(Wrapping(0)),
        }
    }
}

impl<const L: u32> Default for MarkBitvector<L> {
    fn default() -> Self {
        Self::new_unmarked()
    }
}

impl<const L: u32> TypedEq for ThreeValuedBitvector<L> {
    type Output = ThreeValuedBitvector<1>;
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<1>;

    fn typed_eq(
        normal_input: (Self, Self),
        _: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        // every unknown bit may be responsible
        let extended = MarkBitvector(crate::MachineExt::sext(mark_later.0));
        /*(
            extended.limit(normal_input.0),
            extended.limit(normal_input.1),
        )*/
        (extended, extended)
    }
}

impl<const L: u32> Neg for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn neg(normal_input: (Self,), _: Self, mark_later: Self::Mark) -> (Self::Mark,) {
        // TODO: improve, just mark everything for now

        //(Self::Mark::new_marked().limit(normal_input.0),)
        (mark_later,)
    }
}

impl<const L: u32> Add for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn add(
        normal_input: (Self, Self),
        _: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        // TODO: improve, just mark everything for now

        /*(
            Self::Mark::new_marked().limit(normal_input.0),
            Self::Mark::new_marked().limit(normal_input.1),
        )*/
        (Self::Mark::new_marked(), Self::Mark::new_marked())
    }
}
impl<const L: u32> Sub for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn sub(
        normal_input: (Self, Self),
        _: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        // TODO: improve, just mark everything for now

        /*(
            Self::Mark::new_marked().limit(normal_input.0),
            Self::Mark::new_marked().limit(normal_input.1),
        )*/

        (Self::Mark::new_marked(), Self::Mark::new_marked())
    }
}

impl<const L: u32> Mul for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn mul(
        normal_input: (Self, Self),
        _: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        // TODO: improve, just mark everything for now
        /*(
            Self::Mark::new_marked().limit(normal_input.0),
            Self::Mark::new_marked().limit(normal_input.1),
        )*/
        (Self::Mark::new_marked(), Self::Mark::new_marked())
    }
}

impl<const L: u32> Not for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn not(normal_input: (Self,), _: Self, mark_later: Self::Mark) -> (Self::Mark,) {
        // propagate marking of given bits with limitation
        //(mark_later.limit(normal_input.0),)
        (mark_later,)
    }
}

impl<const L: u32> BitAnd for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn bitand(
        normal_input: (Self, Self),
        _: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        /*(
            mark_later.limit(normal_input.0),
            mark_later.limit(normal_input.1),
        )*/
        (mark_later, mark_later)
    }
}
impl<const L: u32> BitOr for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn bitor(
        normal_input: (Self, Self),
        _: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        let result = (
            mark_later.limit(normal_input.0),
            mark_later.limit(normal_input.1),
        );
        println!("Bitor {:?}, {:?}: {:?}", normal_input, mark_later, result);
        result

        //(mark_later, mark_later)
    }
}
impl<const L: u32> BitXor for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn bitxor(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        /*(
            mark_later.limit(normal_input.0),
            mark_later.limit(normal_input.1),
        )*/
        (mark_later, mark_later)
    }
}

impl<const L: u32> TypedCmp for ThreeValuedBitvector<L> {
    type Output = ThreeValuedBitvector<1>;
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<1>;

    fn typed_sgt(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_ugt(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_sgte(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_ugte(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_slt(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_ult(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_slte(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn typed_ulte(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }
}

impl<const L: u32, const X: u32> MachineExt<X> for ThreeValuedBitvector<L> {
    type Output = ThreeValuedBitvector<X>;
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<X>;

    fn uext(
        normal_input: (Self,),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier,) {
        // unsigned extension does not add any bit
        // propagate marking of given bits with limitation
        let extended = MarkBitvector(crate::MachineExt::uext(mark_later.0));
        //(extended.limit(normal_input.0),)
        (extended,)
    }

    fn sext(
        normal_input: (Self,),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier,) {
        // signed extension copies high bit
        // copy it in marking with signed extension
        let extended = MarkBitvector(crate::MachineExt::sext(mark_later.0));
        //(extended.limit(normal_input.0),)
        (extended,)
    }
}

impl<const L: u32> MachineShift for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn sll(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn srl(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn sra(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        todo!()
    }
}
