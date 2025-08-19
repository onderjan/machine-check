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
            let old_mask = util::compute_u64_mask(self.width);
            let new_mask = util::compute_u64_mask(new_width);
            let lengthening_mask = !old_mask & new_mask;
            value |= lengthening_mask;
        }
        RConcreteBitvector::from_masked_u64(value, new_width)
    }
}

impl<const L: u32, const X: u32> Ext<X> for ConcreteBitvector<L> {
    type Output = ConcreteBitvector<X>;

    fn uext(self) -> Self::Output {
        self.to_runtime().uext(X).unwrap_typed()
    }

    fn sext(self) -> Self::Output {
        self.to_runtime().sext(X).unwrap_typed()
    }
}
