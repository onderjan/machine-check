use crate::{
    bitvector::interval::{SignlessInterval, UnsignedInterval},
    concr::ConcreteBitvector,
    forward::{HwArith, HwShift},
    misc::BitvectorBound,
};

use super::DualInterval;

impl<B: BitvectorBound> HwShift for DualInterval<B> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        assert_eq!(self.bound(), amount.bound());
        let bound = self.bound();
        let width = bound.width();

        // shift left is problematic, as it can wrap
        // resolve it using multiplication by (1 << i)

        let amount = amount.to_unsigned_interval();
        let mut intervals = Vec::new();

        for i in amount.min().to_u64()..=amount.max().to_u64() {
            if i >= width as u64 {
                // zero if the shift is too big
                intervals.push(UnsignedInterval::<B>::new(
                    ConcreteBitvector::zero(bound).as_unsigned(),
                    ConcreteBitvector::zero(bound).as_unsigned(),
                ));
                break;
            }

            let value = ConcreteBitvector::one(bound).logic_shl(ConcreteBitvector::new(i, bound));

            let shifted = self.mul(DualInterval::from_value(value));
            intervals.push(shifted.near_half.into_unsigned());
            intervals.push(shifted.far_half.into_unsigned());
        }

        Self::from_unsigned_intervals(intervals)
    }

    fn logic_shr(self, amount: Self) -> Self {
        // logical shift right gives us unsigned intervals
        let mut near_result = None;
        let mut far_result = None;

        let min_shr_value = amount.to_unsigned_interval().min().cast_bitvector();
        let max_shr_value = amount.to_unsigned_interval().max().cast_bitvector();

        if !self.near_half.is_sign_bit_set() {
            let near_min = self.near_half.min().logic_shr(max_shr_value);
            let near_max = self.near_half.max().logic_shr(min_shr_value);

            near_result = Some(UnsignedInterval::new(
                near_min.as_unsigned(),
                near_max.as_unsigned(),
            ));
        }

        if self.far_half.is_sign_bit_set() {
            let far_min = self.far_half.min().logic_shr(max_shr_value);
            let far_max = self.far_half.max().logic_shr(min_shr_value);
            far_result = Some(UnsignedInterval::new(
                far_min.as_unsigned(),
                far_max.as_unsigned(),
            ));
        }

        Self::from_unsigned_intervals([near_result, far_result].into_iter().flatten())
    }

    fn arith_shr(self, amount: Self) -> Self {
        // arithmetic shift right preserves the signs and does not wrap
        let mut near_half = None;
        let mut far_half = None;

        let min_shr_value = amount.to_unsigned_interval().min().cast_bitvector();
        let max_shr_value = amount.to_unsigned_interval().max().cast_bitvector();

        if !self.near_half.is_sign_bit_set() {
            let near_max = self.near_half.max().arith_shr(min_shr_value);
            let near_min = self.near_half.min().arith_shr(max_shr_value);

            near_half = Some(SignlessInterval::new(near_min, near_max));
        }

        if self.far_half.is_sign_bit_set() {
            let far_min = self.far_half.min().arith_shr(min_shr_value);
            let far_max = self.far_half.max().arith_shr(max_shr_value);
            far_half = Some(SignlessInterval::new(far_min, far_max));
        }

        Self::from_opt_halves(near_half, far_half)
    }
}
