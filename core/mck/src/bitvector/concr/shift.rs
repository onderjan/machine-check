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
        let (lhs, amount) = (self.to_runtime(), amount.to_runtime());
        lhs.logic_shl(amount).unwrap_typed()
    }

    fn logic_shr(self, amount: Self) -> Self {
        let (lhs, amount) = (self.to_runtime(), amount.to_runtime());
        lhs.logic_shr(amount).unwrap_typed()
    }

    fn arith_shr(self, amount: Self) -> Self {
        let (lhs, amout) = (self.to_runtime(), amount.to_runtime());
        lhs.arith_shr(amout).unwrap_typed()
    }
}
