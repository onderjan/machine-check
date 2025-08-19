use crate::{concr::RConcreteBitvector, forward::Bitwise};

use super::ConcreteBitvector;

impl RConcreteBitvector {
    pub fn bit_not(self) -> Self {
        Self::from_masked_u64(!self.value, self.width)
    }
    pub fn bit_and(self, rhs: Self) -> Self {
        assert_eq!(self.width, rhs.width);
        Self::from_masked_u64(self.value & rhs.value, self.width)
    }
    pub fn bit_or(self, rhs: Self) -> Self {
        assert_eq!(self.width, rhs.width);
        Self::from_masked_u64(self.value | rhs.value, self.width)
    }
    pub fn bit_xor(self, rhs: Self) -> Self {
        assert_eq!(self.width, rhs.width);
        Self::from_masked_u64(self.value ^ rhs.value, self.width)
    }
}

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
