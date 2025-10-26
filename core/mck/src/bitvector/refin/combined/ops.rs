use crate::{
    abstr::{combined::RCombinedBitvector, three_valued::RThreeValuedBitvector, BitvectorDomain},
    backward::{Bitwise, HwArith, HwShift, RExt, TypedCmp, TypedEq},
    bitvector::refin::{combined::RCombinedMark, three_valued::RMarkBitvector},
    misc::RBound,
    refin::Boolean,
};

impl<R: BitvectorDomain<Bound = RBound>> HwArith for RCombinedBitvector<RThreeValuedBitvector, R> {
    type Mark = RCombinedMark<R>;
    type DivRemResult = (RCombinedMark<R>, RCombinedMark<R>);

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
        mark_later: (RCombinedMark<R>, RCombinedMark<R>),
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwArith>::udiv,
        )
    }

    fn sdiv(
        normal_input: (Self, Self),
        mark_later: (RCombinedMark<R>, RCombinedMark<R>),
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwArith>::sdiv,
        )
    }

    fn urem(
        normal_input: (Self, Self),
        mark_later: (RCombinedMark<R>, RCombinedMark<R>),
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwArith>::urem,
        )
    }

    fn srem(
        normal_input: (Self, Self),
        mark_later: (RCombinedMark<R>, RCombinedMark<R>),
    ) -> (Self::Mark, Self::Mark) {
        Self::divrem_op(
            normal_input,
            mark_later,
            <RThreeValuedBitvector as HwArith>::srem,
        )
    }
}

impl<R: BitvectorDomain<Bound = RBound>> RCombinedBitvector<RThreeValuedBitvector, R> {
    fn uni_op(
        normal_input: (Self,),
        mark_later: RCombinedMark<R>,
        op: fn((RThreeValuedBitvector,), RMarkBitvector) -> (RMarkBitvector,),
    ) -> (RCombinedMark<R>,) {
        let normal_input = (*normal_input.0.left(),);
        let mark_earlier = op(normal_input, mark_later.0);
        (RCombinedMark::new(mark_earlier.0),)
    }

    fn bi_op(
        normal_input: (Self, Self),
        mark_later: RCombinedMark<R>,
        op: fn(
            (RThreeValuedBitvector, RThreeValuedBitvector),
            RMarkBitvector,
        ) -> (RMarkBitvector, RMarkBitvector),
    ) -> (RCombinedMark<R>, RCombinedMark<R>) {
        let normal_input = (*normal_input.0.left(), *normal_input.1.left());
        let mark_earlier = op(normal_input, mark_later.0);
        (
            RCombinedMark::new(mark_earlier.0),
            RCombinedMark::new(mark_earlier.1),
        )
    }

    #[allow(clippy::type_complexity)]
    fn divrem_op(
        normal_input: (Self, Self),
        mark_later: (RCombinedMark<R>, RCombinedMark<R>),
        op: fn(
            (RThreeValuedBitvector, RThreeValuedBitvector),
            (RMarkBitvector, RMarkBitvector),
        ) -> (RMarkBitvector, RMarkBitvector),
    ) -> (RCombinedMark<R>, RCombinedMark<R>) {
        let normal_input = (*normal_input.0.left(), *normal_input.1.left());

        let mark_later = (mark_later.0 .0, mark_later.1 .0);
        let mark_earlier = op(normal_input, mark_later);
        (
            RCombinedMark::new(mark_earlier.0),
            RCombinedMark::new(mark_earlier.1),
        )
    }

    fn cmp_op(
        normal_input: (Self, Self),
        mark_later: Boolean,
        op: fn(
            (RThreeValuedBitvector, RThreeValuedBitvector),
            Boolean,
        ) -> (RMarkBitvector, RMarkBitvector),
    ) -> (RCombinedMark<R>, RCombinedMark<R>) {
        let normal_input = (*normal_input.0.left(), *normal_input.1.left());
        let mark_earlier = op(normal_input, mark_later);
        (
            RCombinedMark::new(mark_earlier.0),
            RCombinedMark::new(mark_earlier.1),
        )
    }

    fn ext_op(
        normal_input: (Self,),
        mark_later: RCombinedMark<R>,
        op: fn((RThreeValuedBitvector,), RMarkBitvector) -> (RMarkBitvector,),
    ) -> (RCombinedMark<R>,) {
        let normal_input = (*normal_input.0.left(),);
        let mark_earlier = op(normal_input, mark_later.0);
        (RCombinedMark::new(mark_earlier.0),)
    }
}

impl<R: BitvectorDomain<Bound = RBound>> Bitwise for RCombinedBitvector<RThreeValuedBitvector, R> {
    type Mark = RCombinedMark<R>;

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

impl<R: BitvectorDomain<Bound = RBound>> TypedCmp for RCombinedBitvector<RThreeValuedBitvector, R> {
    type MarkEarlier = RCombinedMark<R>;
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

impl<R: BitvectorDomain<Bound = RBound>> TypedEq for RCombinedBitvector<RThreeValuedBitvector, R> {
    type MarkEarlier = RCombinedMark<R>;
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

impl<R: BitvectorDomain<Bound = RBound>> RExt for RCombinedBitvector<RThreeValuedBitvector, R> {
    type Mark = RCombinedMark<R>;

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

impl<R: BitvectorDomain<Bound = RBound>> HwShift for RCombinedBitvector<RThreeValuedBitvector, R> {
    type Mark = RCombinedMark<R>;

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
