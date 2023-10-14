use crate::{
    backward::Ext,
    bitvector::{concr, three_valued::abstr::ThreeValuedBitvector},
};

use super::MarkBitvector;

impl<const L: u32, const X: u32> Ext<X> for ThreeValuedBitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<X>;

    fn uext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        // we are going in reverse
        // but unsigned extension does not transport any unknown bit
        // propagate marking of given bits with limitation
        let extended = MarkBitvector(crate::forward::Ext::uext(mark_later.0));
        (extended.limit(normal_input.0),)
    }

    fn sext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        // we are going in reverse

        // in case forward signed extension cut the bitvector or did not do anything,
        // the there was no transport of any unknown bit

        // in case forward signed extension really extended the bitvector, new high bits were added
        // as a copy of the sign bit, propagate marking from these high bits back to the sign bit

        // do unsigned extension and then treat the potential high bits specially

        let mut extended = crate::forward::Ext::<L>::uext(mark_later.0);

        if X > L {
            let back = MarkBitvector(crate::forward::Ext::<X>::uext(extended));
            if mark_later != back {
                // propagate marking to the sign bit
                extended = crate::forward::Bitwise::bitor(extended, concr::Bitvector::bit_mask());
            }
        }

        let extended = MarkBitvector(extended);

        (extended.limit(normal_input.0),)
    }
}
