use crate::{backward::TypedCmp, bitvector::abstr::ThreeValuedBitvector, refin::Boolean};

use super::{support::default_bi_mark, MarkBitvector};

impl<const L: u32> TypedCmp for ThreeValuedBitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = Boolean;

    fn slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, mark_later.0)
    }

    fn ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, mark_later.0)
    }

    fn sle(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, mark_later.0)
    }

    fn ule(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, mark_later.0)
    }
}
