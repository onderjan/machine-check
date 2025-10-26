use std::num::NonZeroU8;

use crate::{
    abstr::combined::RCombinedBitvector,
    bitvector::refin::{combined::RCombinedMark, three_valued::RMarkBitvector},
    misc::{MetaEq, RBound},
    refin::{Boolean, RefinementDomain},
};

impl RefinementDomain for RCombinedMark {
    type Bound = RBound;
    type Abstr = RCombinedBitvector;

    fn bound(&self) -> Self::Bound {
        self.0.bound()
    }

    fn new_unmarked(width: u32) -> Self {
        Self(RMarkBitvector::new_unmarked(width))
    }

    fn new_marked(importance: NonZeroU8, width: u32) -> Self {
        Self(RMarkBitvector::new_marked(importance, width))
    }

    fn from_boolean(boolean: Boolean) -> Self {
        Self(RMarkBitvector::from_boolean(boolean))
    }

    fn apply_join(&mut self, other: &Self) {
        self.0.apply_join(&other.0);
    }

    fn to_condition(self) -> Boolean {
        self.0.to_condition()
    }

    fn apply_refin(&mut self, offer: &Self) -> bool {
        self.0.apply_refin(&offer.0)
    }

    fn force_decay(&self, target: &mut RCombinedBitvector) {
        let mut three_valued = *target.three_valued();
        self.0.force_decay(&mut three_valued);

        *target = RCombinedBitvector::combine(three_valued, *target.dual_interval());
    }

    fn importance(&self) -> u8 {
        self.0.importance()
    }

    fn limit(self, abstract_bitvec: &Self::Abstr) -> Self {
        Self(self.0.limit(abstract_bitvec.three_valued()))
    }
}

impl MetaEq for RCombinedMark {
    fn meta_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
