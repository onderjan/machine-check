use std::num::NonZero;

use crate::{
    backward::HwArith,
    bitvector::{
        abstr::RThreeValuedBitvector,
        refin::three_valued::{
            support::{runtime_default_bi_mark, runtime_default_uni_mark},
            RMarkBitvector,
        },
    },
};

impl HwArith for RThreeValuedBitvector {
    type Mark = RMarkBitvector;
    type DivRemResult = (RMarkBitvector, RMarkBitvector);

    fn arith_neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        runtime_default_uni_mark(normal_input, mark_later)
    }

    fn add(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        runtime_default_bi_mark(normal_input, mark_later)
    }

    fn sub(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        runtime_default_bi_mark(normal_input, mark_later)
    }

    fn mul(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        runtime_default_bi_mark(normal_input, mark_later)
    }

    fn udiv(
        normal_input: (Self, Self),
        mark_later: (RMarkBitvector, RMarkBitvector),
    ) -> (Self::Mark, Self::Mark) {
        runtime_divrem_mark(normal_input, mark_later)
    }

    fn sdiv(
        normal_input: (Self, Self),
        mark_later: (RMarkBitvector, RMarkBitvector),
    ) -> (Self::Mark, Self::Mark) {
        runtime_divrem_mark(normal_input, mark_later)
    }

    fn urem(
        normal_input: (Self, Self),
        mark_later: (RMarkBitvector, RMarkBitvector),
    ) -> (Self::Mark, Self::Mark) {
        runtime_divrem_mark(normal_input, mark_later)
    }

    fn srem(
        normal_input: (Self, Self),
        mark_later: (RMarkBitvector, RMarkBitvector),
    ) -> (Self::Mark, Self::Mark) {
        runtime_divrem_mark(normal_input, mark_later)
    }
}

fn runtime_divrem_mark(
    normal_input: (RThreeValuedBitvector, RThreeValuedBitvector),
    mark_later: (RMarkBitvector, RMarkBitvector),
) -> (RMarkBitvector, RMarkBitvector) {
    assert_eq!(normal_input.0.width(), normal_input.1.width());
    let width = normal_input.0.width();

    // first is result, second is panic
    let mark_later_result = mark_later.0;
    let mark_later_panic = mark_later.1;

    // prefer marking panic
    if mark_later_panic.importance() > 0 {
        // this only depends on the divisor, mark just the divisor
        let importance = if let Some(mark_later) = mark_later_result.inner {
            mark_later.importance
        } else {
            NonZero::<u8>::MIN
        };
        return (
            RMarkBitvector::new_unmarked(width),
            RMarkBitvector::new_marked(importance, width).limit(&normal_input.1),
        );
    }

    // no panic marking, mark normally
    runtime_default_bi_mark(normal_input, mark_later_result)
}
