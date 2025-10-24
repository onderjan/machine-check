/*use std::num::NonZeroU8;

use crate::{
    abstr::combined::RCombinedBitvector,
    bitvector::refin::{
        combined::RCombinedMark,
        three_valued::{RBitvectorMark, RMarkBitvector},
    },
    misc::MetaEq,
};

impl RCombinedMark {
    pub fn new_unmarked(width: u32) -> Self {
        Self(RMarkBitvector::new_unmarked(width))
    }

    pub fn new_marked_unimportant(width: u32) -> Self {
        Self(RMarkBitvector::new_marked_unimportant(width))
    }

    pub fn new_marked(importance: NonZeroU8, width: u32) -> Self {
        Self(RMarkBitvector::new_marked(importance, width))
    }

    pub fn is_marked(&self) -> bool {
        self.0.is_marked()
    }

    pub fn limit(&self, abstract_bitvec: RCombinedBitvector) -> RCombinedMark {
        Self(self.0.limit(*abstract_bitvec.three_valued()))
    }

    pub fn get(&self) -> &Option<RBitvectorMark> {
        self.0.get()
    }
}

impl MetaEq for RCombinedMark {
    fn meta_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
*/
