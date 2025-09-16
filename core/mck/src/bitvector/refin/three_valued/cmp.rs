use crate::{
    backward::TypedCmp,
    bitvector::{
        abstr::{RThreeValuedBitvector, ThreeValuedBitvector},
        refin::{
            three_valued::{support::runtime_default_bi_mark, RMarkBitvector},
            FromRefin,
        },
    },
    refin::Boolean,
};

use super::{support::default_bi_mark, MarkBitvector};

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

impl<const W: u32> TypedCmp for ThreeValuedBitvector<W> {
    type MarkEarlier = MarkBitvector<W>;
    type MarkLater = Boolean;

    fn slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, FromRefin::from_refin(mark_later.0))
    }

    fn ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, FromRefin::from_refin(mark_later.0))
    }

    fn sle(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, FromRefin::from_refin(mark_later.0))
    }

    fn ule(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, FromRefin::from_refin(mark_later.0))
    }
}
