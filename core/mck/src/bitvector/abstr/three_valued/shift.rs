use crate::{
    concr::ConcreteBitvector,
    forward::{Bitwise, HwShift},
};

use super::ThreeValuedBitvector;

impl<const W: u32> HwShift for ThreeValuedBitvector<W> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        // shifting left logically, we need to shift in zeros from right
        let zeros_shift_fn = |value: ConcreteBitvector<W>, amount: ConcreteBitvector<W>| {
            let shifted_mask = ConcreteBitvector::<W>::bit_mask().logic_shl(amount);
            Bitwise::bit_or(value.logic_shl(amount), shifted_mask.bit_not())
        };
        let ones_shift_fn =
            |value: ConcreteBitvector<W>, amount: ConcreteBitvector<W>| value.logic_shl(amount);

        shift(&self, &amount, zeros_shift_fn, ones_shift_fn, &Self::new(0))
    }

    fn logic_shr(self, amount: Self) -> Self {
        // shifting right logically, we need to shift in zeros from left
        let zeros_shift_fn = |value: ConcreteBitvector<W>, amount: ConcreteBitvector<W>| {
            let shifted_mask = ConcreteBitvector::<W>::bit_mask().logic_shr(amount);
            Bitwise::bit_or(value.logic_shr(amount), shifted_mask.bit_not())
        };
        let ones_shift_fn =
            |value: ConcreteBitvector<W>, amount: ConcreteBitvector<W>| value.logic_shr(amount);

        shift(&self, &amount, zeros_shift_fn, ones_shift_fn, &Self::new(0))
    }

    fn arith_shr(self, amount: Self) -> Self {
        // shifting right arithmetically, we need to shift in the sign bit from left
        let sra_shift_fn = |value: ConcreteBitvector<W>, amount: ConcreteBitvector<W>| {
            if value.is_sign_bit_set() {
                let shifted_mask = ConcreteBitvector::<W>::bit_mask().logic_shr(amount);
                Bitwise::bit_or(value.logic_shr(amount), shifted_mask.bit_not())
            } else {
                value.logic_shr(amount)
            }
        };

        // the overflow value is determined by sign bit
        let overflow_zeros = if self.is_zeros_sign_bit_set() {
            ConcreteBitvector::<W>::bit_mask()
        } else {
            ConcreteBitvector::<W>::new(0)
        };

        let overflow_ones = if self.is_ones_sign_bit_set() {
            ConcreteBitvector::<W>::bit_mask()
        } else {
            ConcreteBitvector::<W>::new(0)
        };
        let overflow_value = Self::from_zeros_ones(overflow_zeros, overflow_ones);

        shift(&self, &amount, sra_shift_fn, sra_shift_fn, &overflow_value)
    }
}

fn shift<const W: u32>(
    value: &ThreeValuedBitvector<W>,
    amount: &ThreeValuedBitvector<W>,
    zeros_shift_fn: impl Fn(ConcreteBitvector<W>, ConcreteBitvector<W>) -> ConcreteBitvector<W>,
    ones_shift_fn: impl Fn(ConcreteBitvector<W>, ConcreteBitvector<W>) -> ConcreteBitvector<W>,
    overflow_value: &ThreeValuedBitvector<W>,
) -> ThreeValuedBitvector<W> {
    if W == 0 {
        // avoid problems with zero-width bitvectors
        return *value;
    }

    let mut zeros = ConcreteBitvector::new(0);
    let mut ones = ConcreteBitvector::new(0);

    // the shift amount is also three-valued, which poses problems
    // first, if it can be shifted by L or larger value, join by overflow value
    let shift_overflow = amount.umax().to_u64() >= W as u64;
    if shift_overflow {
        zeros = zeros.bit_or(overflow_value.zeros);
        ones = ones.bit_or(overflow_value.ones);
    }

    // only consider the amounts smaller than L afterwards
    let min_shift = amount.umin().to_u64().min((W - 1) as u64);
    let max_shift = amount.umax().to_u64().min((W - 1) as u64);
    // join by the other shifts iteratively
    for i in min_shift..=max_shift {
        let bi = ConcreteBitvector::new(i);
        if amount.contains_concr(&bi) {
            let shifted_zeros = zeros_shift_fn(value.zeros, bi);
            let shifted_ones = ones_shift_fn(value.ones, bi);
            zeros = zeros.bit_or(shifted_zeros);
            ones = ones.bit_or(shifted_ones);
        }
    }
    ThreeValuedBitvector::from_zeros_ones(zeros, ones)
}
