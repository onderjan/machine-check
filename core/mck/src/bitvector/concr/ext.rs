use crate::{bitvector::util, concr::RConcreteBitvector, forward::Ext};

use super::ConcreteBitvector;

impl RConcreteBitvector {
    pub fn uext(self, new_width: u32) -> RConcreteBitvector {
        // shorten or lengthen as needed
        RConcreteBitvector::from_masked_u64(self.value, new_width)
    }

    pub fn sext(self, new_width: u32) -> RConcreteBitvector {
        let mut value = self.value;
        // copy sign bit to higher positions
        if self.is_sign_bit_set() {
            value |= util::compute_u64_mask(self.width);
        }
        RConcreteBitvector::from_masked_u64(value, new_width)
    }
}

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
