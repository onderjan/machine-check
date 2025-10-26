use crate::{
    abstr::{BitvectorDomain, Boolean},
    forward::{Bitwise, TypedEq},
    misc::BitvectorBound,
};

use super::DualInterval;

impl<B: BitvectorBound> TypedEq for DualInterval<B> {
    type Output = Boolean;
    fn eq(self, rhs: Self) -> Self::Output {
        // result can be true if the intervals have an intersection
        // result can be false if both are not the same concrete value

        let can_be_different = if let (Some(lhs_value), Some(rhs_value)) =
            (self.concrete_value(), rhs.concrete_value())
        {
            lhs_value != rhs_value
        } else {
            true
        };

        let can_be_same = self.meet(&rhs).is_some();

        Self::Output::from_bools(can_be_different, can_be_same)
    }

    fn ne(self, rhs: Self) -> Self::Output {
        self.eq(rhs).bit_not()
    }
}
