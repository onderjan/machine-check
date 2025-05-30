use crate::{
    backward::{Bitwise, Ext, HwArith, HwShift, TypedCmp, TypedEq},
    bitvector::{
        abstr::{CombinedBitvector, ThreeValuedBitvector},
        refin::three_valued::MarkBitvector,
    },
    refin::{Boolean, PanicResult},
};

use super::CombinedMark;

impl<const L: u32> HwArith for CombinedBitvector<L> {
    type Mark = CombinedMark<L>;

    fn arith_neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        Self::uni_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as HwArith>::arith_neg,
        )
    }

    fn add(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as HwArith>::add,
        )
    }

    fn sub(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as HwArith>::sub,
        )
    }

    fn mul(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as HwArith>::mul,
        )
    }

    fn udiv(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as HwArith>::udiv,
        )
    }

    fn sdiv(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as HwArith>::sdiv,
        )
    }

    fn urem(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as HwArith>::urem,
        )
    }

    fn srem(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as HwArith>::srem,
        )
    }
}

impl<const L: u32> CombinedBitvector<L> {
    fn uni_op(
        normal_input: (Self,),
        mark_later: CombinedMark<L>,
        op: fn((ThreeValuedBitvector<L>,), MarkBitvector<L>) -> (MarkBitvector<L>,),
    ) -> (CombinedMark<L>,) {
        let normal_input = (*normal_input.0.three_valued(),);
        let mark_earlier = op(normal_input, mark_later.0);
        (CombinedMark(mark_earlier.0),)
    }

    fn bi_op(
        normal_input: (Self, Self),
        mark_later: CombinedMark<L>,
        op: fn(
            (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
            MarkBitvector<L>,
        ) -> (MarkBitvector<L>, MarkBitvector<L>),
    ) -> (CombinedMark<L>, CombinedMark<L>) {
        let normal_input = (
            *normal_input.0.three_valued(),
            *normal_input.1.three_valued(),
        );
        let mark_earlier = op(normal_input, mark_later.0);
        (CombinedMark(mark_earlier.0), CombinedMark(mark_earlier.1))
    }

    #[allow(clippy::type_complexity)]
    fn divrem_op(
        normal_input: (Self, Self),
        mark_later: PanicResult<CombinedMark<L>>,
        op: fn(
            (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
            PanicResult<MarkBitvector<L>>,
        ) -> (MarkBitvector<L>, MarkBitvector<L>),
    ) -> (CombinedMark<L>, CombinedMark<L>) {
        let normal_input = (
            *normal_input.0.three_valued(),
            *normal_input.1.three_valued(),
        );
        let mark_later = PanicResult {
            panic: mark_later.panic,
            result: mark_later.result.0,
        };

        let mark_earlier = op(normal_input, mark_later);
        (CombinedMark(mark_earlier.0), CombinedMark(mark_earlier.1))
    }

    fn cmp_op(
        normal_input: (Self, Self),
        mark_later: Boolean,
        op: fn(
            (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
            Boolean,
        ) -> (MarkBitvector<L>, MarkBitvector<L>),
    ) -> (CombinedMark<L>, CombinedMark<L>) {
        let normal_input = (
            *normal_input.0.three_valued(),
            *normal_input.1.three_valued(),
        );
        let mark_earlier = op(normal_input, mark_later);
        (CombinedMark(mark_earlier.0), CombinedMark(mark_earlier.1))
    }

    fn ext_op<const X: u32>(
        normal_input: (Self,),
        mark_later: CombinedMark<X>,
        op: fn((ThreeValuedBitvector<L>,), MarkBitvector<X>) -> (MarkBitvector<L>,),
    ) -> (CombinedMark<L>,) {
        let normal_input = (*normal_input.0.three_valued(),);
        let mark_earlier = op(normal_input, mark_later.0);
        (CombinedMark(mark_earlier.0),)
    }
}

impl<const L: u32> Bitwise for CombinedBitvector<L> {
    type Mark = CombinedMark<L>;

    fn bit_not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        Self::uni_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as Bitwise>::bit_not,
        )
    }

    fn bit_and(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as Bitwise>::bit_and,
        )
    }

    fn bit_or(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as Bitwise>::bit_or,
        )
    }

    fn bit_xor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as Bitwise>::bit_xor,
        )
    }
}

impl<const L: u32> TypedCmp for CombinedBitvector<L> {
    type MarkEarlier = CombinedMark<L>;
    type MarkLater = Boolean;

    fn slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as TypedCmp>::slt,
        )
    }

    fn ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as TypedCmp>::ult,
        )
    }

    fn sle(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as TypedCmp>::sle,
        )
    }

    fn ule(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as TypedCmp>::ule,
        )
    }
}

impl<const L: u32> TypedEq for CombinedBitvector<L> {
    type MarkEarlier = CombinedMark<L>;
    type MarkLater = Boolean;

    fn eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as TypedEq>::eq,
        )
    }

    fn ne(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as TypedEq>::ne,
        )
    }
}

impl<const L: u32, const X: u32> Ext<X> for CombinedBitvector<L> {
    type MarkEarlier = CombinedMark<L>;
    type MarkLater = CombinedMark<X>;

    fn uext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        Self::ext_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as Ext<X>>::uext,
        )
    }

    fn sext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        Self::ext_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as Ext<X>>::sext,
        )
    }
}

impl<const L: u32> HwShift for CombinedBitvector<L> {
    type Mark = CombinedMark<L>;

    fn logic_shl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as HwShift>::logic_shl,
        )
    }

    fn logic_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as HwShift>::logic_shr,
        )
    }

    fn arith_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<L> as HwShift>::arith_shr,
        )
    }
}
