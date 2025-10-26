use crate::{
    backward::TypedCmp,
    bitvector::{
        abstr::three_valued::RThreeValuedBitvector,
        refin::three_valued::{support::runtime_default_bi_mark, RMarkBitvector},
    },
    refin::{Boolean, RefinementDomain},
};

impl TypedCmp for RThreeValuedBitvector {
    type MarkEarlier = RMarkBitvector;
    type MarkLater = Boolean;

    fn slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        runtime_default_bi_mark(normal_input, Self::MarkEarlier::from_boolean(mark_later))
    }

    fn ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        runtime_default_bi_mark(normal_input, Self::MarkEarlier::from_boolean(mark_later))
    }

    fn sle(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        runtime_default_bi_mark(normal_input, Self::MarkEarlier::from_boolean(mark_later))
    }

    fn ule(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        runtime_default_bi_mark(normal_input, Self::MarkEarlier::from_boolean(mark_later))
    }
}
