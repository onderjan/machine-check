use crate::{
    bitvector::abstr::CombinedBitvector,
    refin::{Boolean, Refine},
};

use super::CombinedMark;

impl<const L: u32> Refine<CombinedBitvector<L>> for CombinedMark<L> {
    fn apply_join(&mut self, other: &Self) {
        todo!()
    }

    fn to_condition(&self) -> Boolean {
        todo!()
    }

    fn apply_refin(&mut self, offer: &Self) -> bool {
        todo!()
    }

    fn force_decay(&self, target: &mut CombinedBitvector<L>) {
        todo!()
    }

    fn clean() -> Self {
        todo!()
    }

    fn dirty() -> Self {
        todo!()
    }

    fn importance(&self) -> u8 {
        todo!()
    }
}
