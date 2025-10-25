use crate::{bitvector::bound::BitvectorBound, forward::Bitwise};

use super::ConcreteBitvector;

impl<B: BitvectorBound> Bitwise for ConcreteBitvector<B> {
    fn bit_not(self) -> Self {
        Self::from_masked_u64(!self.value, self.bound)
    }
    fn bit_and(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        Self::from_masked_u64(self.value & rhs.value, self.bound)
    }
    fn bit_or(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        Self::from_masked_u64(self.value | rhs.value, self.bound)
    }
    fn bit_xor(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        Self::from_masked_u64(self.value ^ rhs.value, self.bound)
    }
}
