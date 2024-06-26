use crate::{
    backward::TypedEq, bitvector::three_valued::abstr::ThreeValuedBitvector, forward,
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
        // every unknown bit may be responsible
        let extended = MarkBitvector(forward::Ext::sext(mark_later.0 .0));
        (
            extended.limit(normal_input.0),
            extended.limit(normal_input.1),
        )
    }

    fn ne(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        // every unknown bit may be responsible
        let extended = MarkBitvector(forward::Ext::sext(mark_later.0 .0));
        (
            extended.limit(normal_input.0),
            extended.limit(normal_input.1),
        )
    }
}
