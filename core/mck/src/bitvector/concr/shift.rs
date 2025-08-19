use crate::{concr::RConcreteBitvector, forward::HwShift};

use super::ConcreteBitvector;

impl RConcreteBitvector {
    pub fn logic_shl(self, amount: Self) -> Self {
        assert_eq!(self.width, amount.width);
        if amount.value >= self.width as u64 {
            // zero if the shift is too big
            RConcreteBitvector::from_masked_u64(0, self.width)
        } else {
            // apply mask after shifting
            let res = self.value << amount.value;
            RConcreteBitvector::from_masked_u64(res, self.width)
        }
    }

    pub fn logic_shr(self, amount: Self) -> Self {
        assert_eq!(self.width, amount.width);
        if amount.value >= self.width as u64 {
            // zero if the shift is too big
            RConcreteBitvector::from_masked_u64(0, self.width)
        } else {
            RConcreteBitvector::from_masked_u64(self.value >> amount.value, self.width)
        }
    }

    pub fn arith_shr(self, amount: Self) -> Self {
        assert_eq!(self.width, amount.width);
        if amount.value >= self.width as u64 {
            // fill with sign bit if the shift is too big
            if self.is_sign_bit_set() {
                return RConcreteBitvector::from_masked_u64(!0u64, self.width);
            }
            return RConcreteBitvector::from_masked_u64(0, self.width);
        };

        let mut result = self.value >> amount.value;
        // copy sign bit if necessary
        if self.is_sign_bit_set() {
            let old_mask = self.bit_mask_u64();
            let new_mask = old_mask >> amount.value;
            let sign_bit_copy_mask = old_mask & !new_mask;
            result |= sign_bit_copy_mask;
        }
        RConcreteBitvector::from_masked_u64(result, self.width)
    }
}

impl<const L: u32> HwShift for ConcreteBitvector<L> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        if amount.0 >= L as u64 {
            // zero if the shift is too big
            ConcreteBitvector::new(0)
        } else {
            // apply mask after shifting
            let res = self.0 << (amount.0);
            ConcreteBitvector::new(res & Self::bit_mask().0)
        }
    }

    fn logic_shr(self, amount: Self) -> Self {
        if amount.0 >= L as u64 {
            // zero if the shift is too big
            ConcreteBitvector::new(0)
        } else {
            ConcreteBitvector::new(self.0 >> amount.0)
        }
    }

    fn arith_shr(self, amount: Self) -> Self {
        if amount.0 >= L as u64 {
            // fill with sign bit if the shift is too big
            if self.is_sign_bit_set() {
                return ConcreteBitvector::new(Self::bit_mask().0);
            }
            return ConcreteBitvector::new(0);
        };

        let mut result = self.0 >> amount.0;
        // copy sign bit if necessary
        if self.is_sign_bit_set() {
            let old_mask = Self::bit_mask().0;
            let new_mask = old_mask >> amount.0;
            let sign_bit_copy_mask = old_mask & !new_mask;
            result |= sign_bit_copy_mask;
        }
        ConcreteBitvector::new(result)
    }
}
