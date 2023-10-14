use crate::{backward::HwArith, bitvector::three_valued::abstr::ThreeValuedBitvector};

use super::{
    support::{default_bi_mark, default_uni_mark},
    MarkBitvector,
};

impl<const L: u32> HwArith for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

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

    fn udiv(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }

    fn sdiv(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }

    fn urem(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }

    fn srem(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        default_bi_mark(normal_input, mark_later)
    }
}
