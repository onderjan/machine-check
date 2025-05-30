use crate::{bitvector::abstr::CombinedBitvector, traits::misc::Meta};

use super::CombinedMark;

impl<const L: u32> Meta<CombinedBitvector<L>> for CombinedMark<L> {
    fn proto_first(&self) -> CombinedBitvector<L> {
        CombinedBitvector::from_three_valued(self.0.proto_first())
    }

    fn proto_increment(&self, proto: &mut CombinedBitvector<L>) -> bool {
        let mut three_valued = *proto.three_valued();

        let result = self.0.proto_increment(&mut three_valued);
        *proto = CombinedBitvector::from_three_valued(three_valued);
        result
    }
}
