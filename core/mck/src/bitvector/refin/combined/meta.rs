use crate::{bitvector::abstr::CombinedBitvector, traits::misc::Meta};

use super::CombinedMark;

impl<const L: u32> Meta<CombinedBitvector<L>> for CombinedMark<L> {
    fn proto_first(&self) -> CombinedBitvector<L> {
        todo!()
    }

    fn proto_increment(&self, proto: &mut CombinedBitvector<L>) -> bool {
        todo!()
    }
}
