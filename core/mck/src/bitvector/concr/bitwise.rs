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
        self.to_runtime().bit_not().unwrap_typed()
    }
    fn bit_and(self, rhs: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.bit_and(rhs).unwrap_typed()
    }
    fn bit_or(self, rhs: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.bit_or(rhs).unwrap_typed()
    }
    fn bit_xor(self, rhs: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.bit_xor(rhs).unwrap_typed()
    }
}
