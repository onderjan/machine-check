use crate::{bitvector::abstr::CombinedBitvector, traits::misc::Meta};

use super::CombinedMark;

impl<const W: u32> Meta<CombinedBitvector<W>> for CombinedMark<W> {
    fn proto_first(&self) -> CombinedBitvector<W> {
        CombinedBitvector::from_three_valued(self.0.proto_first())
    }

    fn proto_increment(&self, proto: &mut CombinedBitvector<W>) -> bool {
        let mut three_valued = *proto.three_valued();

        let result = self.0.proto_increment(&mut three_valued);
        *proto = CombinedBitvector::from_three_valued(three_valued);
        result
    }
}
