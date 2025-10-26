use std::{marker::PhantomData, num::NonZeroU8};

use crate::{
    abstr::{
        combined::{DomainCombination, RCombinedBitvector, TVDICombination},
        three_valued::RThreeValuedBitvector,
    },
    bitvector::refin::{combined::RCombinedMark, three_valued::RMarkBitvector},
    misc::{MetaEq, RBound},
    refin::{Boolean, RefinementDomain},
};

impl<D: DomainCombination<RBound, Left = RThreeValuedBitvector>> RefinementDomain
    for RCombinedMark<D>
{
    type Bound = RBound;
    type Abstr = RCombinedBitvector<TVDICombination<RBound>>;

    fn bound(&self) -> Self::Bound {
        self.0.bound()
    }

    fn new_unmarked(width: u32) -> Self {
        Self::new(RMarkBitvector::new_unmarked(width))
    }

    fn new_marked(importance: NonZeroU8, width: u32) -> Self {
        Self::new(RMarkBitvector::new_marked(importance, width))
    }

    fn from_boolean(boolean: Boolean) -> Self {
        Self::new(RMarkBitvector::from_boolean(boolean))
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

    fn force_decay(&self, target: &mut Self::Abstr) {
        let mut three_valued = *target.left();
        self.0.force_decay(&mut three_valued);

        *target = RCombinedBitvector::combine(three_valued, *target.right());
    }

    fn importance(&self) -> u8 {
        self.0.importance()
    }

    fn limit(self, abstract_bitvec: &Self::Abstr) -> Self {
        Self::new(self.0.limit(abstract_bitvec.left()))
    }
}

impl<D: DomainCombination<RBound, Left = RThreeValuedBitvector>> RCombinedMark<D> {
    pub(super) fn new(inner: RMarkBitvector) -> Self {
        Self(inner, PhantomData)
    }
}

impl<D: DomainCombination<RBound, Left = RThreeValuedBitvector>> MetaEq for RCombinedMark<D> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
