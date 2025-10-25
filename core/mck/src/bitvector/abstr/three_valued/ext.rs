use crate::{
    bitvector::{bound::CBound, BitvectorBound},
    concr::ConcreteBitvector,
    forward::{BExt, Ext},
};

use super::ThreeValuedBitvector;

impl<B: BitvectorBound, X: BitvectorBound> BExt<X> for ThreeValuedBitvector<B> {
    type Output = ThreeValuedBitvector<X>;

    fn uext(self, new_bound: X) -> Self::Output {
        let old_mask = self.bound().mask();
        let new_mask = new_bound.mask();

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
        ThreeValuedBitvector::from_zeros_ones(
            ConcreteBitvector::new(zeros, new_bound),
            ConcreteBitvector::new(ones, new_bound),
        )
    }

    fn sext(self, new_bound: X) -> Self::Output {
        if self.bound().width() == 0 {
            // no zeros nor ones, handle specially by returning zero
            return ThreeValuedBitvector::new(0, new_bound);
        }

        let old_mask = self.bound().mask();
        let new_mask = new_bound.mask();

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

        ThreeValuedBitvector::from_zeros_ones(
            ConcreteBitvector::new(zeros, new_bound),
            ConcreteBitvector::new(ones, new_bound),
        )
    }
}

impl<const W: u32, const X: u32> Ext<X> for ThreeValuedBitvector<CBound<W>> {
    type Output = ThreeValuedBitvector<CBound<X>>;

    fn uext(self) -> Self::Output {
        BExt::uext(self, CBound::<X>)
    }

    fn sext(self) -> Self::Output {
        BExt::sext(self, CBound::<X>)
    }
}
