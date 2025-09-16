use crate::{bitvector::util, concr::RConcreteBitvector, forward::Ext};

use super::{RThreeValuedBitvector, ThreeValuedBitvector};

impl RThreeValuedBitvector {
    fn uext(self, new_width: u32) -> RThreeValuedBitvector {
        let old_mask = self.bit_mask_u64();
        let new_mask = util::compute_u64_mask(new_width);

        // shorten if needed
        let shortened_zeros = self.zeros.to_u64() & new_mask;
        let shortened_ones = self.ones.to_u64() & new_mask;

        // the mask for lengthening is comprised of bits
        // that were not in the old mask but are in the new mask
        let lengthening_mask = !old_mask & new_mask;

        // for lengthening, we need to add zeros
        let zeros = shortened_zeros | lengthening_mask;
        let ones = shortened_ones;

        // shorten if needed, lengthening is fine
        RThreeValuedBitvector::from_zeros_ones(
            RConcreteBitvector::new(zeros, new_width),
            RConcreteBitvector::new(ones, new_width),
        )
    }

    fn sext(self, new_width: u32) -> RThreeValuedBitvector {
        if self.width() == 0 {
            // no zeros nor ones, handle specially by returning zero
            return RThreeValuedBitvector::new(0, new_width);
        }

        let old_mask = self.bit_mask_u64();
        let new_mask = util::compute_u64_mask(new_width);

        // shorten if needed
        let shortened_zeros = self.zeros.to_u64() & new_mask;
        let shortened_ones = self.ones.to_u64() & new_mask;

        // the mask for lengthening is comprised of bits
        // that were not in the old mask but are in the new mask
        let lengthening_mask = !old_mask & new_mask;

        // for lengthening, we need to extend whatever may be in the sign bit
        let zeros = if self.is_zeros_sign_bit_set() {
            shortened_zeros | lengthening_mask
        } else {
            shortened_zeros
        };

        let ones = if self.is_ones_sign_bit_set() {
            shortened_ones | lengthening_mask
        } else {
            shortened_ones
        };

        RThreeValuedBitvector::from_zeros_ones(
            RConcreteBitvector::new(zeros, new_width),
            RConcreteBitvector::new(ones, new_width),
        )
    }
}

impl<const W: u32, const X: u32> Ext<X> for ThreeValuedBitvector<W> {
    type Output = ThreeValuedBitvector<X>;

    fn uext(self) -> Self::Output {
        self.to_runtime().uext(X).unwrap_typed()
    }

    fn sext(self) -> Self::Output {
        self.to_runtime().sext(X).unwrap_typed()
    }
}
