use crate::{
    concr::{ConcreteBitvector, SignlessInterval, UnsignedInterval},
    forward::{HwArith, HwShift},
};

use super::DualInterval;

impl<const W: u32> HwShift for DualInterval<W> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        // shift left is problematic, as it can wrap
        // resolve it using multiplication by (1 << i)

        let amount = amount.to_unsigned_interval();
        let mut intervals = Vec::new();

        for i in amount.min().to_u64()..=amount.max().to_u64() {
            if i >= W as u64 {
                // zero if the shift is too big
                intervals.push(UnsignedInterval::<W>::new(
                    ConcreteBitvector::zero().cast_unsigned(),
                    ConcreteBitvector::zero().cast_unsigned(),
                ));
                break;
            }

            let value = ConcreteBitvector::one().logic_shl(ConcreteBitvector::new(i));

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

        let min_shr_value = amount.to_unsigned_interval().min().as_bitvector();
        let max_shr_value = amount.to_unsigned_interval().max().as_bitvector();

        if !self.near_half.is_sign_bit_set() {
            let near_min = self.near_half.min().logic_shr(max_shr_value);
            let near_max = self.near_half.max().logic_shr(min_shr_value);

            near_result = Some(UnsignedInterval::new(
                near_min.cast_unsigned(),
                near_max.cast_unsigned(),
            ));
        }

        if self.far_half.is_sign_bit_set() {
            let far_min = self.far_half.min().logic_shr(max_shr_value);
            let far_max = self.far_half.max().logic_shr(min_shr_value);
            far_result = Some(UnsignedInterval::new(
                far_min.cast_unsigned(),
                far_max.cast_unsigned(),
            ));
        }

        Self::from_unsigned_intervals([near_result, far_result].into_iter().flatten())
    }

    fn arith_shr(self, amount: Self) -> Self {
        // arithmetic shift right preserves the signs and does not wrap
        let mut near_half = None;
        let mut far_half = None;

        let min_shr_value = amount.to_unsigned_interval().min().as_bitvector();
        let max_shr_value = amount.to_unsigned_interval().max().as_bitvector();

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
