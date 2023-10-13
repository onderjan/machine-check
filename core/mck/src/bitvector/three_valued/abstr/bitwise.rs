use crate::forward::Bitwise;

use super::ThreeValuedBitvector;

impl<const L: u32> Bitwise for ThreeValuedBitvector<L> {
    fn not(self) -> Self {
        // logical negation
        // swap zeros and ones
        let zeros = self.ones;
        let ones = self.zeros;
        Self::from_zeros_ones(zeros, ones)
    }
    fn bitand(self, rhs: Self) -> Self {
        // logical AND
        // zeros ... if zeros of either are set
        // ones ... only if ones of both are set
        let zeros = self.zeros.bitor(rhs.zeros);
        let ones = self.ones.bitand(rhs.ones);
        Self::from_zeros_ones(zeros, ones)
    }
    fn bitor(self, rhs: Self) -> Self {
        // logical OR
        // zeros ... only if zeros of both are set
        // ones ... if ones of either are set
        let zeros = self.zeros.bitand(rhs.zeros);
        let ones = self.ones.bitor(rhs.ones);
        Self::from_zeros_ones(zeros, ones)
    }
    fn bitxor(self, rhs: Self) -> Self {
        // logical XOR
        // zeros ... if exactly zero or exactly two can be set (both zeros set or both ones set)
        // ones ... if exactly one can be set (lhs zero set and rhs one set or rhs zero set and lhs one set)
        let zeros = (self.zeros.bitand(rhs.zeros)).bitor(self.ones.bitand(rhs.ones));
        let ones = (self.zeros.bitand(rhs.ones)).bitor(self.ones.bitand(rhs.zeros));
        Self::from_zeros_ones(zeros, ones)
    }
}
