use crate::{bitvector::interval::SignlessInterval, forward::Bitwise};

use super::DualInterval;

impl<const L: u32> Bitwise for DualInterval<L> {
    fn bit_not(self) -> Self {
        // just bit-not and swap intervals
        let near_min = self.far_half.max().bit_not();
        let near_max = self.far_half.min().bit_not();
        let far_min = self.near_half.max().bit_not();
        let far_max = self.near_half.min().bit_not();

        Self {
            near_half: SignlessInterval::new(near_min, near_max),
            far_half: SignlessInterval::new(far_min, far_max),
        }
    }
    fn bit_and(self, rhs: Self) -> Self {
        Self::resolve_by_unsigned(self, rhs, |a, b| a.bit_and(b))
    }
    fn bit_or(self, rhs: Self) -> Self {
        Self::resolve_by_unsigned(self, rhs, |a, b| a.bit_or(b))
    }
    fn bit_xor(self, rhs: Self) -> Self {
        Self::resolve_by_unsigned(self, rhs, |a, b| a.bit_xor(b))
    }
}
