use crate::{bitvector::concrete::ConcreteBitvector, forward::TypedCmp};

use super::ThreeValuedBitvector;

impl<const L: u32> TypedCmp for ThreeValuedBitvector<L> {
    type Output = ThreeValuedBitvector<1>;

    fn typed_ult(self, rhs: Self) -> Self::Output {
        // use unsigned versions
        let lhs_min = self.umin().as_unsigned();
        let lhs_max = self.umax().as_unsigned();
        let rhs_min = rhs.umin().as_unsigned();
        let rhs_max = rhs.umax().as_unsigned();

        // can be zero if lhs can be greater or equal to rhs
        // this is only possible if lhs max can be greater or equal to rhs min
        let result_can_be_zero = lhs_max >= rhs_min;

        // can be one if lhs can be lesser than rhs
        // this is only possible if lhs min can be lesser than rhs max
        let result_can_be_one = lhs_min < rhs_max;

        Self::Output::from_zeros_ones(
            ConcreteBitvector::new(result_can_be_zero as u64),
            ConcreteBitvector::new(result_can_be_one as u64),
        )
    }

    fn typed_ulte(self, rhs: Self) -> Self::Output {
        // use unsigned versions
        let lhs_min = self.umin().as_unsigned();
        let lhs_max = self.umax().as_unsigned();
        let rhs_min = rhs.umin().as_unsigned();
        let rhs_max = rhs.umax().as_unsigned();

        // can be zero if lhs can be greater than rhs
        // this is only possible if lhs max can be greater to rhs min
        let result_can_be_zero = lhs_max > rhs_min;

        // can be one if lhs can be lesser or equal to rhs
        // this is only possible if lhs min can be lesser or equal to rhs max
        let result_can_be_one = lhs_min <= rhs_max;

        Self::Output::from_zeros_ones(
            ConcreteBitvector::new(result_can_be_zero as u64),
            ConcreteBitvector::new(result_can_be_one as u64),
        )
    }

    fn typed_slt(self, rhs: Self) -> Self::Output {
        // use signed versions
        let lhs_min = self.smin().as_signed();
        let lhs_max = self.smax().as_signed();
        let rhs_min = rhs.smin().as_signed();
        let rhs_max = rhs.smax().as_signed();

        // can be zero if lhs can be greater or equal to rhs
        // this is only possible if lhs max can be greater or equal to rhs min
        let result_can_be_zero = lhs_max >= rhs_min;

        // can be one if lhs can be lesser than rhs
        // this is only possible if lhs min can be lesser than rhs max
        let result_can_be_one = lhs_min < rhs_max;

        Self::Output::from_zeros_ones(
            ConcreteBitvector::new(result_can_be_zero as u64),
            ConcreteBitvector::new(result_can_be_one as u64),
        )
    }

    fn typed_slte(self, rhs: Self) -> Self::Output {
        // use signed versions
        let lhs_min = self.smin().as_signed();
        let lhs_max = self.smax().as_signed();
        let rhs_min = rhs.smin().as_signed();
        let rhs_max = rhs.smax().as_signed();

        // can be zero if lhs can be greater than rhs
        // this is only possible if lhs max can be greater to rhs min
        let result_can_be_zero = lhs_max > rhs_min;

        // can be one if lhs can be lesser or equal to rhs
        // this is only possible if lhs min can be lesser or equal to rhs max
        let result_can_be_one = lhs_min <= rhs_max;

        Self::Output::from_zeros_ones(
            ConcreteBitvector::new(result_can_be_zero as u64),
            ConcreteBitvector::new(result_can_be_one as u64),
        )
    }
}
