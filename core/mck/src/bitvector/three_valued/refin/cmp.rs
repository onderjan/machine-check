use crate::{backward::TypedCmp, bitvector::three_valued::abstr::ThreeValuedBitvector};

use super::{support::default_bi_mark, MarkBitvector};

impl<const L: u32> TypedCmp for ThreeValuedBitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<1>;

    fn typed_slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, mark_later)
    }

    fn typed_ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, mark_later)
    }

    fn typed_slte(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, mark_later)
    }

    fn typed_ulte(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        default_bi_mark(normal_input, mark_later)
    }
}
