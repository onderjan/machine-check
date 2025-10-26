use crate::{
    abstr::combined::RCombinedBitvector, bitvector::refin::combined::RCombinedMark,
    traits::misc::Meta,
};

impl Meta<RCombinedBitvector> for RCombinedMark {
    fn proto_first(&self) -> RCombinedBitvector {
        RCombinedBitvector::from_three_valued(self.0.proto_first())
    }

    fn proto_increment(&self, proto: &mut RCombinedBitvector) -> bool {
        let mut three_valued = *proto.three_valued();

        let result = self.0.proto_increment(&mut three_valued);
        *proto = RCombinedBitvector::from_three_valued(three_valued);
        result
    }
}
