use crate::{
    bitvector::{concrete::ConcreteBitvector, util},
    forward::Ext,
};

use super::ThreeValuedBitvector;

impl<const L: u32, const X: u32> Ext<X> for ThreeValuedBitvector<L> {
    type Output = ThreeValuedBitvector<X>;

    fn uext(self) -> Self::Output {
        let old_mask = util::compute_u64_mask(L);
        let new_mask = util::compute_u64_mask(X);

        // shorten if needed
        let shortened_zeros = self.zeros.as_unsigned() & new_mask;
        let shortened_ones = self.ones.as_unsigned() & new_mask;

        // the mask for lengthening is comprised of bits
        // that were not in the old mask but are in the new mask
        let lengthening_mask = !old_mask & new_mask;

        // for lengthening, we need to add zeros
        let zeros = shortened_zeros | lengthening_mask;
        let ones = shortened_ones;

        // shorten if needed, lengthening is fine
        Self::Output::from_zeros_ones(ConcreteBitvector::new(zeros), ConcreteBitvector::new(ones))
    }

    fn sext(self) -> Self::Output {
        if L == 0 {
            // no zeros nor ones, handle specially by returning zero
            return Self::Output::new(0);
        }

        let old_mask = util::compute_u64_mask(L);
        let new_mask = util::compute_u64_mask(X);

        // shorten if needed
        let shortened_zeros = self.zeros.as_unsigned() & new_mask;
        let shortened_ones = self.ones.as_unsigned() & new_mask;

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

        Self::Output::from_zeros_ones(ConcreteBitvector::new(zeros), ConcreteBitvector::new(ones))
    }
}
