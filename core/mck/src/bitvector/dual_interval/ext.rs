use crate::{
    bitvector::concrete::{SignedInterval, UnsignedInterval},
    forward::Ext,
};

use super::DualInterval;

impl<const L: u32, const X: u32> Ext<X> for DualInterval<L> {
    type Output = DualInterval<X>;

    fn uext(self) -> DualInterval<X> {
        let near: UnsignedInterval<X> = self.near_half.into_unsigned().ext();
        let far: UnsignedInterval<X> = self.far_half.into_unsigned().ext();

        DualInterval::from_unsigned_intervals([near, far])
    }

    fn sext(self) -> DualInterval<X> {
        let near: SignedInterval<X> = self.near_half.into_signed().ext();
        let far: SignedInterval<X> = self.far_half.into_signed().ext();

        DualInterval::from_signed_intervals(&[near, far])
    }
}
