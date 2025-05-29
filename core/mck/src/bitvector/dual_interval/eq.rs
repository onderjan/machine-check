use crate::{
    abstr::Boolean,
    bitvector::concrete::ConcreteBitvector,
    forward::{Bitwise, TypedEq},
};

use super::DualInterval;

impl<const W: u32> TypedEq for DualInterval<W> {
    type Output = Boolean;
    fn eq(self, rhs: Self) -> Self::Output {
        // result can be true if the intervals have an intersection
        // result can be false if both are not the same concrete value

        let intersection = self.intersection(&rhs);

        let can_be_same = intersection.is_some();
        let can_be_different = if let (Some(lhs_value), Some(rhs_value)) =
            (self.concrete_value(), rhs.concrete_value())
        {
            lhs_value != rhs_value
        } else {
            true
        };

        Self::Output::from_zeros_ones(
            ConcreteBitvector::new(can_be_different as u64),
            ConcreteBitvector::new(can_be_same as u64),
        )
    }

    fn ne(self, rhs: Self) -> Self::Output {
        self.eq(rhs).bit_not()
    }
}
