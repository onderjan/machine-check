use crate::{
    backward::Bitwise,
    bitvector::{abstr::three_valued::RThreeValuedBitvector, refin::three_valued::RMarkBitvector},
};

impl Bitwise for RThreeValuedBitvector {
    type Mark = RMarkBitvector;

    fn bit_not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        // propagate marking of given bits with limitation
        (mark_later.limit(&normal_input.0),)
    }

    fn bit_and(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        (
            mark_later.limit(&normal_input.0),
            mark_later.limit(&normal_input.1),
        )
    }

    fn bit_or(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        (
            mark_later.limit(&normal_input.0),
            mark_later.limit(&normal_input.1),
        )
    }

    fn bit_xor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        (
            mark_later.limit(&normal_input.0),
            mark_later.limit(&normal_input.1),
        )
    }
}
