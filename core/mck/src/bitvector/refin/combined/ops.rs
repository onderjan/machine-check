use crate::{
    backward::{Bitwise, Ext, HwArith, HwShift, TypedCmp, TypedEq},
    bitvector::{
        abstr::{CombinedBitvector, ThreeValuedBitvector},
        refin::three_valued::MarkBitvector,
    },
    refin::{Boolean, PanicResult},
};

use super::CombinedMark;

impl<const W: u32> HwArith for CombinedBitvector<W> {
    type Mark = CombinedMark<W>;

    fn arith_neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        Self::uni_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as HwArith>::arith_neg,
        )
    }

    fn add(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as HwArith>::add,
        )
    }

    fn sub(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as HwArith>::sub,
        )
    }

    fn mul(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as HwArith>::mul,
        )
    }

    fn udiv(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as HwArith>::udiv,
        )
    }

    fn sdiv(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as HwArith>::sdiv,
        )
    }

    fn urem(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as HwArith>::urem,
        )
    }

    fn srem(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as HwArith>::srem,
        )
    }
}

impl<const W: u32> CombinedBitvector<W> {
    fn uni_op(
        normal_input: (Self,),
        mark_later: CombinedMark<W>,
        op: fn((ThreeValuedBitvector<W>,), MarkBitvector<W>) -> (MarkBitvector<W>,),
    ) -> (CombinedMark<W>,) {
        let normal_input = (*normal_input.0.three_valued(),);
        let mark_earlier = op(normal_input, mark_later.0);
        (CombinedMark(mark_earlier.0),)
    }

    fn bi_op(
        normal_input: (Self, Self),
        mark_later: CombinedMark<W>,
        op: fn(
            (ThreeValuedBitvector<W>, ThreeValuedBitvector<W>),
            MarkBitvector<W>,
        ) -> (MarkBitvector<W>, MarkBitvector<W>),
    ) -> (CombinedMark<W>, CombinedMark<W>) {
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
        mark_later: PanicResult<CombinedMark<W>>,
        op: fn(
            (ThreeValuedBitvector<W>, ThreeValuedBitvector<W>),
            PanicResult<MarkBitvector<W>>,
        ) -> (MarkBitvector<W>, MarkBitvector<W>),
    ) -> (CombinedMark<W>, CombinedMark<W>) {
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
            (ThreeValuedBitvector<W>, ThreeValuedBitvector<W>),
            Boolean,
        ) -> (MarkBitvector<W>, MarkBitvector<W>),
    ) -> (CombinedMark<W>, CombinedMark<W>) {
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
        op: fn((ThreeValuedBitvector<W>,), MarkBitvector<X>) -> (MarkBitvector<W>,),
    ) -> (CombinedMark<W>,) {
        let normal_input = (*normal_input.0.three_valued(),);
        let mark_earlier = op(normal_input, mark_later.0);
        (CombinedMark(mark_earlier.0),)
    }
}

impl<const W: u32> Bitwise for CombinedBitvector<W> {
    type Mark = CombinedMark<W>;

    fn bit_not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        Self::uni_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as Bitwise>::bit_not,
        )
    }

    fn bit_and(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as Bitwise>::bit_and,
        )
    }

    fn bit_or(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as Bitwise>::bit_or,
        )
    }

    fn bit_xor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as Bitwise>::bit_xor,
        )
    }
}

impl<const W: u32> TypedCmp for CombinedBitvector<W> {
    type MarkEarlier = CombinedMark<W>;
    type MarkLater = Boolean;

    fn slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as TypedCmp>::slt,
        )
    }

    fn ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as TypedCmp>::ult,
        )
    }

    fn sle(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as TypedCmp>::sle,
        )
    }

    fn ule(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as TypedCmp>::ule,
        )
    }
}

impl<const W: u32> TypedEq for CombinedBitvector<W> {
    type MarkEarlier = CombinedMark<W>;
    type MarkLater = Boolean;

    fn eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as TypedEq>::eq,
        )
    }

    fn ne(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as TypedEq>::ne,
        )
    }
}

impl<const W: u32, const X: u32> Ext<X> for CombinedBitvector<W> {
    type MarkEarlier = CombinedMark<W>;
    type MarkLater = CombinedMark<X>;

    fn uext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        Self::ext_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as Ext<X>>::uext,
        )
    }

    fn sext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        Self::ext_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as Ext<X>>::sext,
        )
    }
}

impl<const W: u32> HwShift for CombinedBitvector<W> {
    type Mark = CombinedMark<W>;

    fn logic_shl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as HwShift>::logic_shl,
        )
    }

    fn logic_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as HwShift>::logic_shr,
        )
    }

    fn arith_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <ThreeValuedBitvector<W> as HwShift>::arith_shr,
        )
    }
}
