use crate::{abstr::Boolean, forward::TypedCmp};

use super::Bitvector;

impl<const L: u32> TypedCmp for Bitvector<L> {
    type Output = Boolean;

    fn ult(self, rhs: Self) -> Self::Output {
        let lhs_interval = self.unsigned_interval();
        let rhs_interval = rhs.unsigned_interval();
        let can_be_false = !lhs_interval.all_pairs_lt(rhs_interval);
        let can_be_true = lhs_interval.some_pairs_lt(rhs_interval);

        Boolean::from_bools(can_be_false, can_be_true)
    }

    fn ule(self, rhs: Self) -> Self::Output {
        let lhs_interval = self.unsigned_interval();
        let rhs_interval = rhs.unsigned_interval();
        let can_be_false = !lhs_interval.all_pairs_lte(rhs_interval);
        let can_be_true = lhs_interval.some_pairs_lte(rhs_interval);

        Boolean::from_bools(can_be_false, can_be_true)
    }

    fn slt(self, rhs: Self) -> Self::Output {
        let lhs_interval = self.offset_signed_interval();
        let rhs_interval = rhs.offset_signed_interval();
        let can_be_false = !lhs_interval.all_pairs_lt(rhs_interval);
        let can_be_true = lhs_interval.some_pairs_lt(rhs_interval);

        Boolean::from_bools(can_be_false, can_be_true)
    }

    fn sle(self, rhs: Self) -> Self::Output {
        let lhs_interval = self.offset_signed_interval();
        let rhs_interval = rhs.offset_signed_interval();
        let can_be_false = !lhs_interval.all_pairs_lte(rhs_interval);
        let can_be_true = lhs_interval.some_pairs_lte(rhs_interval);

        Boolean::from_bools(can_be_false, can_be_true)
    }
}
