use crate::{
    bitvector::abstr::three_valued::RThreeValuedBitvector,
    concr::RConcreteBitvector,
    forward::{Bitwise, HwShift},
};

use super::ThreeValuedBitvector;

impl HwShift for RThreeValuedBitvector {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        assert_eq!(self.width(), amount.width());

        // shifting left logically, we need to shift in zeros from right
        let zeros_shift_fn = |value: RConcreteBitvector, amount: RConcreteBitvector| {
            let shifted_mask = self.bit_mask_bitvector().logic_shl(amount);
            Bitwise::bit_or(value.logic_shl(amount), shifted_mask.bit_not())
        };
        let ones_shift_fn =
            |value: RConcreteBitvector, amount: RConcreteBitvector| value.logic_shl(amount);

        shift(
            &self,
            &amount,
            zeros_shift_fn,
            ones_shift_fn,
            &Self::new(0, self.width()),
        )
    }

    fn logic_shr(self, amount: Self) -> Self {
        assert_eq!(self.width(), amount.width());

        // shifting right logically, we need to shift in zeros from left
        let zeros_shift_fn = |value: RConcreteBitvector, amount: RConcreteBitvector| {
            let shifted_mask = self.bit_mask_bitvector().logic_shr(amount);
            Bitwise::bit_or(value.logic_shr(amount), shifted_mask.bit_not())
        };
        let ones_shift_fn =
            |value: RConcreteBitvector, amount: RConcreteBitvector| value.logic_shr(amount);

        shift(
            &self,
            &amount,
            zeros_shift_fn,
            ones_shift_fn,
            &Self::new(0, self.width()),
        )
    }

    fn arith_shr(self, amount: Self) -> Self {
        assert_eq!(self.width(), amount.width());

        // shifting right arithmetically, we need to shift in the sign bit from left
        let sra_shift_fn = |value: RConcreteBitvector, amount: RConcreteBitvector| {
            if value.is_sign_bit_set() {
                let shifted_mask = self.bit_mask_bitvector().logic_shr(amount);
                Bitwise::bit_or(value.logic_shr(amount), shifted_mask.bit_not())
            } else {
                value.logic_shr(amount)
            }
        };

        // the overflow value is determined by sign bit
        let overflow_zeros = if self.is_zeros_sign_bit_set() {
            self.bit_mask_bitvector()
        } else {
            RConcreteBitvector::new(0, self.width())
        };

        let overflow_ones = if self.is_ones_sign_bit_set() {
            self.bit_mask_bitvector()
        } else {
            RConcreteBitvector::new(0, self.width())
        };
        let overflow_value = Self::from_zeros_ones(overflow_zeros, overflow_ones);

        shift(&self, &amount, sra_shift_fn, sra_shift_fn, &overflow_value)
    }
}

impl<const W: u32> HwShift for ThreeValuedBitvector<W> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), amount.to_runtime());
        lhs.logic_shl(rhs).unwrap_typed()
    }

    fn logic_shr(self, amount: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), amount.to_runtime());
        lhs.logic_shr(rhs).unwrap_typed()
    }

    fn arith_shr(self, amount: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), amount.to_runtime());
        lhs.arith_shr(rhs).unwrap_typed()
    }
}

fn shift(
    value: &RThreeValuedBitvector,
    amount: &RThreeValuedBitvector,
    zeros_shift_fn: impl Fn(RConcreteBitvector, RConcreteBitvector) -> RConcreteBitvector,
    ones_shift_fn: impl Fn(RConcreteBitvector, RConcreteBitvector) -> RConcreteBitvector,
    overflow_value: &RThreeValuedBitvector,
) -> RThreeValuedBitvector {
    assert_eq!(value.width(), amount.width());
    let width = value.width();
    if width == 0 {
        // avoid problems with zero-width bitvectors
        return *value;
    }

    let mut zeros = RConcreteBitvector::new(0, width);
    let mut ones = RConcreteBitvector::new(0, width);

    // the shift amount is also three-valued, which poses problems
    // first, if it can be shifted by L or larger value, join by overflow value
    let shift_overflow = amount.umax().to_u64() >= width as u64;
    if shift_overflow {
        zeros = zeros.bit_or(overflow_value.zeros);
        ones = ones.bit_or(overflow_value.ones);
    }

    // only consider the amounts smaller than L afterwards
    let min_shift = amount.umin().to_u64().min((width - 1) as u64);
    let max_shift = amount.umax().to_u64().min((width - 1) as u64);
    // join by the other shifts iteratively
    for i in min_shift..=max_shift {
        let bi = RConcreteBitvector::new(i, width);
        if amount.contains_concr(&bi) {
            let shifted_zeros = zeros_shift_fn(value.zeros, bi);
            let shifted_ones = ones_shift_fn(value.ones, bi);
            zeros = zeros.bit_or(shifted_zeros);
            ones = ones.bit_or(shifted_ones);
        }
    }
    RThreeValuedBitvector::from_zeros_ones(zeros, ones)
}
