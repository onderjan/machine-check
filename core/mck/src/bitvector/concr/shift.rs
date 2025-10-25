use crate::{bitvector::BitvectorBound, forward::HwShift};

use super::ConcreteBitvector;

impl<B: BitvectorBound> HwShift for ConcreteBitvector<B> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        assert_eq!(self.bound, amount.bound);
        if amount.value >= self.bound.width() as u64 {
            // zero if the shift is too big
            ConcreteBitvector::from_masked_u64(0, self.bound)
        } else {
            // apply mask after shifting
            let res = self.value << amount.value;
            ConcreteBitvector::from_masked_u64(res, self.bound)
        }
    }

    fn logic_shr(self, amount: Self) -> Self {
        assert_eq!(self.bound, amount.bound);
        if amount.value >= self.bound.width() as u64 {
            // zero if the shift is too big
            ConcreteBitvector::from_masked_u64(0, self.bound)
        } else {
            ConcreteBitvector::from_masked_u64(self.value >> amount.value, self.bound)
        }
    }

    fn arith_shr(self, amount: Self) -> Self {
        assert_eq!(self.bound, amount.bound);
        if amount.value >= self.bound.width() as u64 {
            // fill with sign bit if the shift is too big
            if self.is_sign_bit_set() {
                return ConcreteBitvector::from_masked_u64(!0u64, self.bound);
            }
            return ConcreteBitvector::from_masked_u64(0, self.bound);
        };

        let mut result = self.value >> amount.value;
        // copy sign bit if necessary
        if self.is_sign_bit_set() {
            let old_mask = self.bound.mask();
            let new_mask = old_mask >> amount.value;
            let sign_bit_copy_mask = old_mask & !new_mask;
            result |= sign_bit_copy_mask;
        }
        ConcreteBitvector::from_masked_u64(result, self.bound)
    }
}
