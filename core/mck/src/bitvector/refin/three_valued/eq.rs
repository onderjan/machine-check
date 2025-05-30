use crate::{
    backward::TypedEq,
    bitvector::{abstr::ThreeValuedBitvector, refin::FromRefin},
    forward,
    refin::Boolean,
};

use super::MarkBitvector;

impl<const L: u32> TypedEq for ThreeValuedBitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = Boolean;

    fn eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        let bv_later: MarkBitvector<1> = FromRefin::from_refin(mark_later.0);

        let Some(mark_later) = bv_later.0 else {
            return (MarkBitvector::new_unmarked(), MarkBitvector::new_unmarked());
        };

        // every unknown bit may be responsible
        // copy importance
        let extended =
            MarkBitvector::new(forward::Ext::sext(mark_later.mark), mark_later.importance);
        (
            extended.limit(normal_input.0),
            extended.limit(normal_input.1),
        )
    }

    fn ne(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        let bv_later: MarkBitvector<1> = FromRefin::from_refin(mark_later.0);

        let Some(mark_later) = bv_later.0 else {
            return (MarkBitvector::new_unmarked(), MarkBitvector::new_unmarked());
        };

        // every unknown bit may be responsible
        // copy importance
        let extended =
            MarkBitvector::new(forward::Ext::sext(mark_later.mark), mark_later.importance);
        (
            extended.limit(normal_input.0),
            extended.limit(normal_input.1),
        )
    }
}
