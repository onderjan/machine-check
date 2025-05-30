use crate::{abstr::Boolean, forward::TypedCmp};

use super::DualInterval;

impl<const L: u32> TypedCmp for DualInterval<L> {
    type Output = crate::abstr::Boolean;

    fn ult(self, rhs: Self) -> Self::Output {
        // use unsigned versions
        let lhs = self.to_unsigned_interval();
        let rhs = rhs.to_unsigned_interval();

        // can be false if lhs can be greater or equal to rhs
        // this is only possible if lhs max can be greater or equal to rhs min
        let result_can_be_false = lhs.max() >= rhs.min();

        // can be true if lhs can be lesser than rhs
        // this is only possible if lhs min can be lesser than rhs max
        let result_can_be_true = lhs.min() < rhs.max();

        Boolean::from_bools(result_can_be_false, result_can_be_true)
    }

    fn ule(self, rhs: Self) -> Self::Output {
        // use unsigned versions
        let lhs = self.to_unsigned_interval();
        let rhs = rhs.to_unsigned_interval();

        // can be false if lhs can be greater than rhs
        // this is only possible if lhs max can be greater to rhs min
        let result_can_be_false = lhs.max() > rhs.min();

        // can be true if lhs can be lesser or equal to rhs
        // this is only possible if lhs min can be lesser or equal to rhs max
        let result_can_be_true = lhs.min() <= rhs.max();

        Boolean::from_bools(result_can_be_false, result_can_be_true)
    }

    fn slt(self, rhs: Self) -> Self::Output {
        // use signed versions
        let lhs = self.to_signed_interval();
        let rhs = rhs.to_signed_interval();

        // can be false if lhs can be greater or equal to rhs
        // this is only possible if lhs max can be greater or equal to rhs min
        let result_can_be_false = lhs.max() >= rhs.min();

        // can be true if lhs can be lesser than rhs
        // this is only possible if lhs min can be lesser than rhs max
        let result_can_be_true = lhs.min() < rhs.max();

        Boolean::from_bools(result_can_be_false, result_can_be_true)
    }

    fn sle(self, rhs: Self) -> Self::Output {
        // use signed versions
        let lhs = self.to_signed_interval();
        let rhs = rhs.to_signed_interval();

        // can be false if lhs can be greater than rhs
        // this is only possible if lhs max can be greater to rhs min
        let result_can_be_false = lhs.max() > rhs.min();

        // can be true if lhs can be lesser or equal to rhs
        // this is only possible if lhs min can be lesser or equal to rhs max
        let result_can_be_true = lhs.min() <= rhs.max();

        Boolean::from_bools(result_can_be_false, result_can_be_true)
    }
}
