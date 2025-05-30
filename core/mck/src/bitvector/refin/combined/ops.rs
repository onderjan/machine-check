use crate::{
    backward::{Bitwise, Ext, HwArith, HwShift, TypedCmp, TypedEq},
    bitvector::abstr::CombinedBitvector,
    refin::{Boolean, PanicResult},
};

use super::CombinedMark;

impl<const L: u32> HwArith for CombinedBitvector<L> {
    type Mark = CombinedMark<L>;

    fn arith_neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        todo!()
    }

    fn add(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn sub(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn mul(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn udiv(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn sdiv(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn urem(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn srem(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        todo!()
    }
}

impl<const L: u32> Bitwise for CombinedBitvector<L> {
    type Mark = CombinedMark<L>;

    fn bit_not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        todo!()
    }

    fn bit_and(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn bit_or(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn bit_xor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        todo!()
    }
}

impl<const L: u32> TypedCmp for CombinedBitvector<L> {
    type MarkEarlier = CombinedMark<L>;
    type MarkLater = Boolean;

    fn slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn sle(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn ule(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }
}

impl<const L: u32> TypedEq for CombinedBitvector<L> {
    type MarkEarlier = CombinedMark<L>;
    type MarkLater = Boolean;

    fn eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }

    fn ne(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        todo!()
    }
}

impl<const L: u32, const X: u32> Ext<X> for CombinedBitvector<L> {
    type MarkEarlier = CombinedMark<L>;
    type MarkLater = CombinedMark<X>;

    fn uext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        todo!()
    }

    fn sext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        todo!()
    }
}

impl<const L: u32> HwShift for CombinedBitvector<L> {
    type Mark = CombinedMark<L>;

    fn logic_shl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn logic_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        todo!()
    }

    fn arith_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        todo!()
    }
}
