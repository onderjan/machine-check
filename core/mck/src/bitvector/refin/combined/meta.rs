use crate::{
    abstr::{
        combination::DomainCombination, combined::RCombinedBitvector,
        three_valued::RThreeValuedBitvector,
    },
    bitvector::refin::combined::RCombinedMark,
    misc::RBound,
    traits::misc::Meta,
};

impl<D: DomainCombination<RBound, Left = RThreeValuedBitvector>> Meta<RCombinedBitvector<D>>
    for RCombinedMark<D>
{
    fn proto_first(&self) -> RCombinedBitvector<D> {
        RCombinedBitvector::from_left(self.0.proto_first())
    }

    fn proto_increment(&self, proto: &mut RCombinedBitvector<D>) -> bool {
        let mut three_valued = *proto.left();

        let result = self.0.proto_increment(&mut three_valued);
        *proto = RCombinedBitvector::from_left(three_valued);
        result
    }
}
