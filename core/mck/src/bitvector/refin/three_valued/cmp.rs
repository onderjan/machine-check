use crate::{
    backward::TypedCmp,
    bitvector::{
        abstr::three_valued::RThreeValuedBitvector,
        refin::three_valued::{support::runtime_default_bi_mark, RMarkBitvector},
    },
    refin::Boolean,
};

impl TypedCmp for RThreeValuedBitvector {
    type MarkEarlier = RMarkBitvector;
    type MarkLater = Boolean;

    fn slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        runtime_default_bi_mark(normal_input, mark_later.to_runtime_bitvector())
    }

    fn ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        runtime_default_bi_mark(normal_input, mark_later.to_runtime_bitvector())
    }

    fn sle(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        runtime_default_bi_mark(normal_input, mark_later.to_runtime_bitvector())
    }

    fn ule(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        runtime_default_bi_mark(normal_input, mark_later.to_runtime_bitvector())
    }
}
