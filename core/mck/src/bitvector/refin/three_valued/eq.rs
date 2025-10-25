use crate::{
    backward::TypedEq,
    bitvector::{abstr::three_valued::RThreeValuedBitvector, refin::three_valued::RMarkBitvector},
    forward,
    misc::BitvectorBound,
    refin::Boolean,
};

impl TypedEq for RThreeValuedBitvector {
    type MarkEarlier = RMarkBitvector;
    type MarkLater = Boolean;

    fn eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        assert_eq!(normal_input.0.width(), normal_input.1.width());
        let bound = normal_input.0.bound();
        let width = bound.width();

        let bv_later = mark_later.to_runtime_bitvector();

        let Some(mark_later) = bv_later.inner else {
            return (
                RMarkBitvector::new_unmarked(width),
                RMarkBitvector::new_unmarked(width),
            );
        };

        // every unknown bit may be responsible
        // copy importance
        let extended = forward::BExt::sext(mark_later.mark, bound);
        let extended = RMarkBitvector::new(extended, mark_later.importance, width);
        (
            extended.limit(&normal_input.0),
            extended.limit(&normal_input.1),
        )
    }

    fn ne(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        assert_eq!(normal_input.0.width(), normal_input.1.width());
        let bound = normal_input.0.bound();
        let width = bound.width();

        let bv_later = mark_later.to_runtime_bitvector();

        let Some(mark_later) = bv_later.inner else {
            return (
                RMarkBitvector::new_unmarked(width),
                RMarkBitvector::new_unmarked(width),
            );
        };

        // every unknown bit may be responsible
        let extended = forward::BExt::sext(mark_later.mark, bound);
        // copy importance
        let extended = RMarkBitvector::new(extended, mark_later.importance, width);
        (
            extended.limit(&normal_input.0),
            extended.limit(&normal_input.1),
        )
    }
}
