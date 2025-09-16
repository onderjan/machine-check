use crate::{
    backward::Ext,
    bitvector::{
        abstr::{RThreeValuedBitvector, ThreeValuedBitvector},
        concr::ConcreteBitvector,
        refin::three_valued::RMarkBitvector,
    },
    concr::RConcreteBitvector,
};

use super::MarkBitvector;

impl RThreeValuedBitvector {
    pub fn mark_uext(normal_input: (Self,), mark_later: RMarkBitvector) -> (RMarkBitvector,) {
        // normal input and earlier mark have the same width
        // later mark has the extension width
        let earlier_width = normal_input.0.width();

        let Some(mark_later) = mark_later.inner else {
            return (RMarkBitvector::new_unmarked(earlier_width),);
        };

        // we are going in reverse
        // but unsigned extension does not transport any unknown bit
        // propagate marking of given bits with limitation
        let mark_earlier = mark_later.mark.uext(earlier_width);
        let extended = RMarkBitvector::new(mark_earlier, mark_later.importance, earlier_width);
        (extended.limit(normal_input.0),)
    }

    pub fn mark_sext(normal_input: (Self,), mark_later: RMarkBitvector) -> (RMarkBitvector,) {
        // normal input and earlier mark have the same width
        // later mark has the extension width
        let earlier_width = normal_input.0.width();
        let later_width = mark_later.width;

        let Some(mark_later) = mark_later.inner else {
            return (RMarkBitvector::new_unmarked(earlier_width),);
        };

        // we are going in reverse

        // in case forward signed extension cut the bitvector or did not do anything,
        // the there was no transport of any unknown bit

        // in case forward signed extension really extended the bitvector, new high bits were added
        // as a copy of the sign bit, propagate marking from these high bits back to the sign bit

        // do unsigned extension and then treat the potential high bits specially

        let mut extended = mark_later.mark.uext(earlier_width);

        if later_width > earlier_width && mark_later.mark != extended.uext(later_width) {
            // propagate marking to the sign bit
            extended = crate::forward::Bitwise::bit_or(
                extended,
                RConcreteBitvector::bit_mask_bitvector(extended),
            );
        }

        let extended = RMarkBitvector::new(extended, mark_later.importance, earlier_width);

        (extended.limit(normal_input.0),)
    }
}

impl<const W: u32, const X: u32> Ext<X> for ThreeValuedBitvector<W> {
    type MarkEarlier = MarkBitvector<W>;
    type MarkLater = MarkBitvector<X>;

    fn uext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        let Some(mark_later) = mark_later.0 else {
            return (MarkBitvector::new_unmarked(),);
        };

        // we are going in reverse
        // but unsigned extension does not transport any unknown bit
        // propagate marking of given bits with limitation
        let mark_earlier = crate::forward::Ext::uext(mark_later.mark);
        let extended = MarkBitvector::new(mark_earlier, mark_later.importance);
        (extended.limit(normal_input.0),)
    }

    fn sext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        let Some(mark_later) = mark_later.0 else {
            return (MarkBitvector::new_unmarked(),);
        };

        // we are going in reverse

        // in case forward signed extension cut the bitvector or did not do anything,
        // the there was no transport of any unknown bit

        // in case forward signed extension really extended the bitvector, new high bits were added
        // as a copy of the sign bit, propagate marking from these high bits back to the sign bit

        // do unsigned extension and then treat the potential high bits specially

        let mut extended = crate::forward::Ext::<W>::uext(mark_later.mark);

        if X > W && mark_later.mark != crate::forward::Ext::<X>::uext(extended) {
            // propagate marking to the sign bit
            extended = crate::forward::Bitwise::bit_or(extended, ConcreteBitvector::bit_mask());
        }

        let extended = MarkBitvector::new(extended, mark_later.importance);

        (extended.limit(normal_input.0),)
    }
}
