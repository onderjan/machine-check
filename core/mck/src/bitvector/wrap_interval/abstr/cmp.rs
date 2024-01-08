use crate::forward::TypedCmp;

use super::Bitvector;

impl<const L: u32> TypedCmp for Bitvector<L> {
    type Output = Bitvector<1>;

    fn typed_ult(self, rhs: Self) -> Self::Output {
        let lhs_interval = self.unsigned_interval();
        let rhs_interval = rhs.unsigned_interval();
        let can_be_false = !lhs_interval.all_pairs_lt(rhs_interval);
        let can_be_true = lhs_interval.some_pairs_lt(rhs_interval);

        Bitvector::from_bools(can_be_false, can_be_true)
    }

    fn typed_ulte(self, rhs: Self) -> Self::Output {
        let lhs_interval = self.unsigned_interval();
        let rhs_interval = rhs.unsigned_interval();
        let can_be_false = !lhs_interval.all_pairs_lte(rhs_interval);
        let can_be_true = lhs_interval.some_pairs_lte(rhs_interval);

        Bitvector::from_bools(can_be_false, can_be_true)
    }

    fn typed_slt(self, rhs: Self) -> Self::Output {
        let lhs_interval = self.offset_signed_interval();
        let rhs_interval = rhs.offset_signed_interval();
        let can_be_false = !lhs_interval.all_pairs_lt(rhs_interval);
        let can_be_true = lhs_interval.some_pairs_lt(rhs_interval);

        Bitvector::from_bools(can_be_false, can_be_true)
    }

    fn typed_slte(self, rhs: Self) -> Self::Output {
        let lhs_interval = self.offset_signed_interval();
        let rhs_interval = rhs.offset_signed_interval();
        let can_be_false = !lhs_interval.all_pairs_lte(rhs_interval);
        let can_be_true = lhs_interval.some_pairs_lte(rhs_interval);

        Bitvector::from_bools(can_be_false, can_be_true)
    }
}
