use crate::{
    abstr::BitvectorDomain,
    backward::RExt,
    bitvector::{
        abstr::three_valued::RThreeValuedBitvector, refin::three_valued::RMarkBitvector, RBound,
    },
    concr::ConcreteBitvector,
    misc::BitvectorBound,
    refin::RefinementDomain,
    traits::forward,
};

impl RExt for RThreeValuedBitvector {
    type Mark = RMarkBitvector;

    fn uext(normal_input: (Self,), mark_later: RMarkBitvector) -> (RMarkBitvector,) {
        // normal input and earlier mark have the same width
        // later mark has the extension width
        let earlier_bound = normal_input.0.bound();
        let earlier_width = earlier_bound.width();

        let Some(mark_later) = mark_later.inner else {
            return (RMarkBitvector::new_unmarked(earlier_width),);
        };

        // we are going in reverse
        // but unsigned extension does not transport any unknown bit
        // propagate marking of given bits with limitation
        let mark_earlier = forward::BExt::uext(mark_later.mark, earlier_bound);
        let extended = RMarkBitvector::new(mark_earlier, mark_later.importance, earlier_width);
        (extended.limit(&normal_input.0),)
    }

    fn sext(normal_input: (Self,), mark_later: RMarkBitvector) -> (RMarkBitvector,) {
        // normal input and earlier mark have the same width
        // later mark has the extension width
        let earlier_bound = normal_input.0.bound();
        let earlier_width = earlier_bound.width();

        let later_width = mark_later.width;
        let later_bound = RBound::new(later_width);

        let Some(mark_later) = mark_later.inner else {
            return (RMarkBitvector::new_unmarked(earlier_width),);
        };

        // we are going in reverse

        // in case forward signed extension cut the bitvector or did not do anything,
        // the there was no transport of any unknown bit

        // in case forward signed extension really extended the bitvector, new high bits were added
        // as a copy of the sign bit, propagate marking from these high bits back to the sign bit

        // do unsigned extension and then treat the potential high bits specially

        let mut extended = forward::BExt::uext(mark_later.mark, earlier_bound);

        let unextended = forward::BExt::uext(extended, later_bound);

        if later_width > earlier_width && mark_later.mark != unextended {
            // propagate marking to the sign bit
            extended = crate::forward::Bitwise::bit_or(
                extended,
                ConcreteBitvector::bit_mask(earlier_bound),
            );
        }

        let extended = RMarkBitvector::new(extended, mark_later.importance, earlier_width);

        (extended.limit(&normal_input.0),)
    }
}
