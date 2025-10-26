use crate::{
    bitvector::interval::{SignedInterval, UnsignedInterval},
    forward::{BExt, Ext},
    misc::{BitvectorBound, CBound},
};

use super::DualInterval;

impl<B: BitvectorBound, X: BitvectorBound> BExt<X> for DualInterval<B> {
    type Output = DualInterval<X>;

    fn uext(self, new_bound: X) -> DualInterval<X> {
        let near: UnsignedInterval<X> = self.near_half.into_unsigned().ext(new_bound);
        let far: UnsignedInterval<X> = self.far_half.into_unsigned().ext(new_bound);

        DualInterval::from_unsigned_intervals([near, far])
    }

    fn sext(self, new_bound: X) -> DualInterval<X> {
        let near: SignedInterval<X> = self.near_half.into_signed().ext(new_bound);
        let far: SignedInterval<X> = self.far_half.into_signed().ext(new_bound);

        DualInterval::from_signed_intervals(&[near, far])
    }
}

impl<const W: u32, const X: u32> Ext<X> for DualInterval<CBound<W>> {
    type Output = DualInterval<CBound<X>>;

    fn uext(self) -> Self::Output {
        BExt::uext(self, CBound::<X>)
    }

    fn sext(self) -> Self::Output {
        BExt::sext(self, CBound::<X>)
    }
}
