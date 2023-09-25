use crate::{
    mark::{
        Add, BitAnd, BitOr, BitXor, MachineExt, MachineShift, Mul, Neg, Not, Sub, TypedCmp, TypedEq,
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
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }
}

impl<const L: u32> Neg for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn neg(normal_input: (Self,), normal_output: Self, mark_later: Self::Mark) -> (Self::Mark,) {
        todo!()
    }
}

impl<const L: u32> Add for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn add(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        todo!()
    }
}
impl<const L: u32> Sub for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn sub(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        todo!()
    }
}

impl<const L: u32> Mul for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn mul(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        todo!()
    }
}

impl<const L: u32> Not for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn not(normal_input: (Self,), normal_output: Self, mark_later: Self::Mark) -> (Self::Mark,) {
        todo!()
    }
}

impl<const L: u32> BitAnd for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn bitand(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        todo!()
    }
}
impl<const L: u32> BitOr for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn bitor(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        todo!()
    }
}
impl<const L: u32> BitXor for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn bitxor(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark) {
        todo!()
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
        todo!()
    }

    fn sext(
        normal_input: (Self,),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier,) {
        todo!()
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
