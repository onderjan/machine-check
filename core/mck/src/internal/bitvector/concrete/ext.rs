use crate::{bitvector::util, forward::Ext};

use super::ConcreteBitvector;

impl<const L: u32, const X: u32> Ext<X> for ConcreteBitvector<L> {
    type Output = ConcreteBitvector<X>;

    fn uext(self) -> Self::Output {
        // shorten if needed, lengthening is fine
        ConcreteBitvector::<X>::new(self.0 & util::compute_u64_mask(X))
    }

    fn sext(self) -> Self::Output {
        // shorten if needed
        let mut v = self.0 & util::compute_u64_mask(X);
        // copy sign bit if necessary
        if self.is_sign_bit_set() {
            let old_mask = Self::bit_mask().0;
            let new_mask = util::compute_u64_mask(X);
            let lengthening_mask = !old_mask & new_mask;
            v |= lengthening_mask;
        }
        ConcreteBitvector::<X>::new(v)
    }
}
