use crate::{
    bitvector::bound::{BitvectorBound, CBound},
    forward::{BExt, Ext},
};

use super::ConcreteBitvector;

impl<B: BitvectorBound, X: BitvectorBound> BExt<X> for ConcreteBitvector<B> {
    type Output = ConcreteBitvector<X>;

    fn uext(self, new_bound: X) -> ConcreteBitvector<X> {
        // shorten or lengthen as needed
        ConcreteBitvector::from_masked_u64(self.value, new_bound)
    }

    fn sext(self, new_bound: X) -> ConcreteBitvector<X> {
        let mut value = self.value;
        // copy sign bit to higher positions
        if self.is_sign_bit_set() {
            let old_mask = self.bound.mask();
            let new_mask = new_bound.mask();
            let lengthening_mask = !old_mask & new_mask;
            value |= lengthening_mask;
        }
        ConcreteBitvector::from_masked_u64(value, new_bound)
    }
}

impl<const W: u32, const X: u32> Ext<X> for ConcreteBitvector<CBound<W>> {
    type Output = ConcreteBitvector<CBound<X>>;

    fn uext(self) -> Self::Output {
        BExt::uext(self, CBound::<X>)
    }

    fn sext(self) -> Self::Output {
        BExt::sext(self, CBound::<X>)
    }
}
