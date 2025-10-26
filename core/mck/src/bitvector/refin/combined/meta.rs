use crate::{
    abstr::{combined::RCombinedBitvector, three_valued::RThreeValuedBitvector, BitvectorDomain},
    bitvector::refin::combined::RCombinedMark,
    misc::RBound,
    traits::misc::Meta,
};

impl<R: BitvectorDomain<Bound = RBound>> Meta<RCombinedBitvector<RThreeValuedBitvector, R>>
    for RCombinedMark<R>
{
    fn proto_first(&self) -> RCombinedBitvector<RThreeValuedBitvector, R> {
        RCombinedBitvector::from_left(self.0.proto_first())
    }

    fn proto_increment(&self, proto: &mut RCombinedBitvector<RThreeValuedBitvector, R>) -> bool {
        let mut three_valued = *proto.left();

        let result = self.0.proto_increment(&mut three_valued);
        *proto = RCombinedBitvector::from_left(three_valued);
        result
    }
}
