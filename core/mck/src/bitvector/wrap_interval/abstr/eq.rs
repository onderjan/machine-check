use crate::{forward::TypedEq, traits::forward::Bitwise};

use super::Bitvector;

impl<const L: u32> TypedEq for Bitvector<L> {
    type Output = Bitvector<1>;
    fn eq(self, rhs: Self) -> Self::Output {
        // consider all intervals, otherwise the results would be inexact
        let lhs_intervals = self.unsigned_intervals();
        let rhs_intervals = rhs.unsigned_intervals();

        let mut can_be_false = false;
        let mut can_be_true = false;
        for lhs_interval in lhs_intervals.iter() {
            for rhs_interval in rhs_intervals.iter() {
                can_be_false |= !lhs_interval.all_pairs_eq(*rhs_interval);
                can_be_true |= lhs_interval.some_pairs_eq(*rhs_interval);
            }
        }
        Bitvector::<1>::from_bools(can_be_false, can_be_true)
    }

    fn ne(self, rhs: Self) -> Self::Output {
        self.eq(rhs).bit_not()
    }
}
