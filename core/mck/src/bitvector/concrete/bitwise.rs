use crate::forward::Bitwise;

use super::ConcreteBitvector;

impl<const L: u32> Bitwise for ConcreteBitvector<L> {
    fn not(self) -> Self {
        Self::new((!self.0) & Self::bit_mask().0)
    }
    fn bitand(self, rhs: Self) -> Self {
        Self::new((self.0 & rhs.0) & Self::bit_mask().0)
    }
    fn bitor(self, rhs: Self) -> Self {
        Self::new((self.0 | rhs.0) & Self::bit_mask().0)
    }
    fn bitxor(self, rhs: Self) -> Self {
        Self::new((self.0 ^ rhs.0) & Self::bit_mask().0)
    }
}
