use crate::{concr::ConcreteBitvector, forward::TypedCmp};

use super::ThreeValuedBitvector;

impl<const L: u32> TypedCmp for ThreeValuedBitvector<L> {
    type Output = crate::abstr::Boolean;

    fn ult(self, rhs: Self) -> Self::Output {
        // use unsigned versions
        let lhs_min = self.umin();
        let lhs_max = self.umax();
        let rhs_min = rhs.umin();
        let rhs_max = rhs.umax();

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

    fn ule(self, rhs: Self) -> Self::Output {
        // use unsigned versions
        let lhs_min = self.umin();
        let lhs_max = self.umax();
        let rhs_min = rhs.umin();
        let rhs_max = rhs.umax();

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

    fn slt(self, rhs: Self) -> Self::Output {
        // use signed versions
        let lhs_min = self.smin();
        let lhs_max = self.smax();
        let rhs_min = rhs.smin();
        let rhs_max = rhs.smax();

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

    fn sle(self, rhs: Self) -> Self::Output {
        // use signed versions
        let lhs_min = self.smin();
        let lhs_max = self.smax();
        let rhs_min = rhs.smin();
        let rhs_max = rhs.smax();

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
