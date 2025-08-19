use crate::forward::Bitwise;

use super::ThreeValuedBitvector;

impl<const W: u32> Bitwise for ThreeValuedBitvector<W> {
    fn bit_not(self) -> Self {
        // logical negation
        // swap zeros and ones
        let zeros = self.ones;
        let ones = self.zeros;
        Self::from_zeros_ones(zeros, ones)
    }
    fn bit_and(self, rhs: Self) -> Self {
        // logical AND
        // zeros ... if zeros of either are set
        // ones ... only if ones of both are set
        let zeros = self.zeros.bit_or(rhs.zeros);
        let ones = self.ones.bit_and(rhs.ones);
        Self::from_zeros_ones(zeros, ones)
    }
    fn bit_or(self, rhs: Self) -> Self {
        // logical OR
        // zeros ... only if zeros of both are set
        // ones ... if ones of either are set
        let zeros = self.zeros.bit_and(rhs.zeros);
        let ones = self.ones.bit_or(rhs.ones);
        Self::from_zeros_ones(zeros, ones)
    }
    fn bit_xor(self, rhs: Self) -> Self {
        // logical XOR
        // zeros ... if exactly zero or exactly two can be set (both zeros set or both ones set)
        // ones ... if exactly one can be set (lhs zero set and rhs one set or rhs zero set and lhs one set)
        let zeros = (self.zeros.bit_and(rhs.zeros)).bit_or(self.ones.bit_and(rhs.ones));
        let ones = (self.zeros.bit_and(rhs.ones)).bit_or(self.ones.bit_and(rhs.zeros));
        Self::from_zeros_ones(zeros, ones)
    }
}
