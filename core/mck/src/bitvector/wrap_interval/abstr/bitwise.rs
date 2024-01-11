use crate::forward::Bitwise;

use super::Bitvector;

impl<const L: u32> Bitwise for Bitvector<L> {
    fn bit_not(self) -> Self {
        Self::from_wrap_interval(self.end.bit_not(), self.start.bit_not())
    }
    fn bit_and(self, rhs: Self) -> Self {
        let mut intervals = Vec::new();
        for lhs_interval in self.unsigned_intervals() {
            for rhs_interval in rhs.unsigned_intervals() {
                intervals.push(lhs_interval.bit_and(rhs_interval));
            }
        }
        Self::from_intervals(intervals)
    }
    fn bit_or(self, rhs: Self) -> Self {
        let mut intervals = Vec::new();
        for lhs_interval in self.unsigned_intervals() {
            for rhs_interval in rhs.unsigned_intervals() {
                intervals.push(lhs_interval.bit_or(rhs_interval));
            }
        }
        Self::from_intervals(intervals)
    }
    fn bit_xor(self, rhs: Self) -> Self {
        let mut intervals = Vec::new();
        for lhs_interval in self.unsigned_intervals() {
            for rhs_interval in rhs.unsigned_intervals() {
                intervals.push(lhs_interval.bit_xor(rhs_interval));
            }
        }
        Self::from_intervals(intervals)
    }
}
