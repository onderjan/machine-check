use crate::{
    backward::TypedEq,
    bitvector::{
        abstr::{RThreeValuedBitvector, ThreeValuedBitvector},
        refin::{three_valued::RMarkBitvector, FromRefin},
    },
    forward,
    refin::Boolean,
};

use super::MarkBitvector;

impl TypedEq for RThreeValuedBitvector {
    type MarkEarlier = RMarkBitvector;
    type MarkLater = Boolean;

    fn eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        assert_eq!(normal_input.0.width(), normal_input.1.width());
        let width = normal_input.0.width();

        let bv_later = mark_later.to_runtime_bitvector();

        let Some(mark_later) = bv_later.inner else {
            return (
                RMarkBitvector::new_unmarked(width),
                RMarkBitvector::new_unmarked(width),
            );
        };

        // every unknown bit may be responsible
        // copy importance
        let extended =
            RMarkBitvector::new(mark_later.mark.sext(width), mark_later.importance, width);
        (
            extended.limit(normal_input.0),
            extended.limit(normal_input.1),
        )
    }

    fn ne(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        assert_eq!(normal_input.0.width(), normal_input.1.width());
        let width = normal_input.0.width();

        let bv_later = mark_later.to_runtime_bitvector();

        let Some(mark_later) = bv_later.inner else {
            return (
                RMarkBitvector::new_unmarked(width),
                RMarkBitvector::new_unmarked(width),
            );
        };

        // every unknown bit may be responsible
        // copy importance
        let extended =
            RMarkBitvector::new(mark_later.mark.sext(width), mark_later.importance, width);
        (
            extended.limit(normal_input.0),
            extended.limit(normal_input.1),
        )
    }
}

impl<const W: u32> TypedEq for ThreeValuedBitvector<W> {
    type MarkEarlier = MarkBitvector<W>;
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
