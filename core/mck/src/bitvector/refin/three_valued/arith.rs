use std::num::NonZero;

use crate::{backward::HwArith, bitvector::abstr::ThreeValuedBitvector, refin::PanicResult};

use super::{
    support::{default_bi_mark, default_uni_mark},
    MarkBitvector,
};

impl<const W: u32> HwArith for ThreeValuedBitvector<W> {
    type Mark = MarkBitvector<W>;

    fn arith_neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
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

    fn udiv(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        divrem_mark(normal_input, mark_later)
    }

    fn sdiv(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        divrem_mark(normal_input, mark_later)
    }

    fn urem(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        divrem_mark(normal_input, mark_later)
    }

    fn srem(
        normal_input: (Self, Self),
        mark_later: PanicResult<Self::Mark>,
    ) -> (Self::Mark, Self::Mark) {
        divrem_mark(normal_input, mark_later)
    }
}

fn divrem_mark<const W: u32>(
    normal_input: (ThreeValuedBitvector<W>, ThreeValuedBitvector<W>),
    mark_later: PanicResult<MarkBitvector<W>>,
) -> (MarkBitvector<W>, MarkBitvector<W>) {
    let mark_later_panic = mark_later.panic;
    let mark_later_result = mark_later.result;

    // prefer marking panic
    if mark_later_panic.is_marked() {
        // this only depends on the divisor, mark just the divisor
        let importance = if let Some(mark_later) = mark_later.result.0 {
            mark_later.importance
        } else {
            NonZero::<u8>::MIN
        };
        return (
            MarkBitvector::new_unmarked(),
            MarkBitvector::new_marked(importance).limit(normal_input.1),
        );
    }

    // no panic marking, mark normally
    default_bi_mark(normal_input, mark_later_result)
}
