/*use crate::{
    abstr::{combined::RCombinedBitvector, RPanicResult},
    backward::{Bitwise, HwArith, HwShift, RExt, TypedCmp, TypedEq},
    bitvector::{
        abstr::RThreeValuedBitvector,
        refin::{combined::RCombinedMark, three_valued::RMarkBitvector},
    },
    refin::Boolean,
};

impl HwArith for RCombinedBitvector {
    type Mark = RCombinedMark;
    type DivRemResult = RPanicResult<RCombinedMark>;

    fn arith_neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        Self::uni_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwArith>::arith_neg,
        )
    }

    fn add(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwArith>::add,
        )
    }

    fn sub(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwArith>::sub,
        )
    }

    fn mul(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwArith>::mul,
        )
    }

    fn udiv(
        normal_input: (Self, Self),
        mark_later: RPanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwArith>::udiv,
        )
    }

    fn sdiv(
        normal_input: (Self, Self),
        mark_later: RPanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwArith>::sdiv,
        )
    }

    fn urem(
        normal_input: (Self, Self),
        mark_later: RPanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwArith>::urem,
        )
    }

    fn srem(
        normal_input: (Self, Self),
        mark_later: RPanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwArith>::srem,
        )
    }
}

impl RCombinedBitvector {
    fn uni_op(
        normal_input: (Self,),
        mark_later: RCombinedMark,
        op: fn((RThreeValuedBitvector,), RMarkBitvector) -> (RMarkBitvector,),
    ) -> (RCombinedMark,) {
        let normal_input = (*normal_input.0.three_valued(),);
        let mark_earlier = op(normal_input, mark_later.0);
        (RCombinedMark(mark_earlier.0),)
    }

    fn bi_op(
        normal_input: (Self, Self),
        mark_later: RCombinedMark,
        op: fn(
            (RThreeValuedBitvector, RThreeValuedBitvector),
            RMarkBitvector,
        ) -> (RMarkBitvector, RMarkBitvector),
    ) -> (RCombinedMark, RCombinedMark) {
        let normal_input = (
            *normal_input.0.three_valued(),
            *normal_input.1.three_valued(),
        );
        let mark_earlier = op(normal_input, mark_later.0);
        (RCombinedMark(mark_earlier.0), RCombinedMark(mark_earlier.1))
    }

    #[allow(clippy::type_complexity)]
    fn divrem_op(
        normal_input: (Self, Self),
        mark_later: RPanicResult<RCombinedMark>,
        op: fn(
            (RThreeValuedBitvector, RThreeValuedBitvector),
            RPanicResult<RMarkBitvector>,
        ) -> (RMarkBitvector, RMarkBitvector),
    ) -> (RCombinedMark, RCombinedMark) {
        let normal_input = (
            *normal_input.0.three_valued(),
            *normal_input.1.three_valued(),
        );
        let mark_later = RPanicResult {
            panic: mark_later.panic,
            result: mark_later.result.0,
        };

        let mark_earlier = op(normal_input, mark_later);
        (RCombinedMark(mark_earlier.0), RCombinedMark(mark_earlier.1))
    }

    fn cmp_op(
        normal_input: (Self, Self),
        mark_later: Boolean,
        op: fn(
            (RThreeValuedBitvector, RThreeValuedBitvector),
            Boolean,
        ) -> (RMarkBitvector, RMarkBitvector),
    ) -> (RCombinedMark, RCombinedMark) {
        let normal_input = (
            *normal_input.0.three_valued(),
            *normal_input.1.three_valued(),
        );
        let mark_earlier = op(normal_input, mark_later);
        (RCombinedMark(mark_earlier.0), RCombinedMark(mark_earlier.1))
    }

    fn ext_op<const X: u32>(
        normal_input: (Self,),
        mark_later: RCombinedMark,
        op: fn((RThreeValuedBitvector,), RMarkBitvector) -> (RMarkBitvector,),
    ) -> (RCombinedMark,) {
        let normal_input = (*normal_input.0.three_valued(),);
        let mark_earlier = op(normal_input, mark_later.0);
        (RCombinedMark(mark_earlier.0),)
    }
}

impl Bitwise for RCombinedBitvector {
    type Mark = RCombinedMark;

    fn bit_not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        Self::uni_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as Bitwise>::bit_not,
        )
    }

    fn bit_and(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as Bitwise>::bit_and,
        )
    }

    fn bit_or(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as Bitwise>::bit_or,
        )
    }

    fn bit_xor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as Bitwise>::bit_xor,
        )
    }
}

impl TypedCmp for RCombinedBitvector {
    type MarkEarlier = RCombinedMark;
    type MarkLater = Boolean;

    fn slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as TypedCmp>::slt,
        )
    }

    fn ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as TypedCmp>::ult,
        )
    }

    fn sle(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as TypedCmp>::sle,
        )
    }

    fn ule(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as TypedCmp>::ule,
        )
    }
}

impl TypedEq for RCombinedBitvector {
    type MarkEarlier = RCombinedMark;
    type MarkLater = Boolean;

    fn eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as TypedEq>::eq,
        )
    }

    fn ne(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        Self::cmp_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as TypedEq>::ne,
        )
    }
}

impl RExt for RCombinedBitvector {
    type Mark = RCombinedMark;

    fn uext(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        Self::ext_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as RExt>::uext,
        )
    }

    fn sext(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        Self::ext_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as RExt>::sext,
        )
    }
}

impl HwShift for RCombinedBitvector {
    type Mark = RCombinedMark;

    fn logic_shl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwShift>::logic_shl,
        )
    }

    fn logic_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwShift>::logic_shr,
        )
    }

    fn arith_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        Self::bi_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwShift>::arith_shr,
        )
    }
}
*/
