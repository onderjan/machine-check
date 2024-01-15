use crate::forward::Bitwise;

use super::ConcreteBitvector;

impl<const L: u32> Bitwise for ConcreteBitvector<L> {
    fn bit_not(self) -> Self {
        Self::new((!self.0) & Self::bit_mask().0)
    }
    fn bit_and(self, rhs: Self) -> Self {
        Self::new((self.0 & rhs.0) & Self::bit_mask().0)
    }
    fn bit_or(self, rhs: Self) -> Self {
        Self::new((self.0 | rhs.0) & Self::bit_mask().0)
    }
    fn bit_xor(self, rhs: Self) -> Self {
        Self::new((self.0 ^ rhs.0) & Self::bit_mask().0)
    }
}
