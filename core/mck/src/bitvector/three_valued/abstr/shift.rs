use crate::{
    bitvector::concr,
    forward::{Bitwise, HwShift},
};

use super::ThreeValuedBitvector;

impl<const L: u32> HwShift for ThreeValuedBitvector<L> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        // shifting left logically, we need to shift in zeros from right
        let zeros_shift_fn = |value: concr::Bitvector<L>, amount: concr::Bitvector<L>| {
            let shifted_mask = concr::Bitvector::<L>::bit_mask().logic_shl(amount);
            Bitwise::bitor(value.logic_shl(amount), shifted_mask.not())
        };
        let ones_shift_fn =
            |value: concr::Bitvector<L>, amount: concr::Bitvector<L>| value.logic_shl(amount);

        shift(&self, &amount, zeros_shift_fn, ones_shift_fn, &Self::new(0))
    }

    fn logic_shr(self, amount: Self) -> Self {
        // shifting right logically, we need to shift in zeros from left
        let zeros_shift_fn = |value: concr::Bitvector<L>, amount: concr::Bitvector<L>| {
            let shifted_mask = concr::Bitvector::<L>::bit_mask().logic_shr(amount);
            Bitwise::bitor(value.logic_shr(amount), shifted_mask.not())
        };
        let ones_shift_fn =
            |value: concr::Bitvector<L>, amount: concr::Bitvector<L>| value.logic_shr(amount);

        shift(&self, &amount, zeros_shift_fn, ones_shift_fn, &Self::new(0))
    }

    fn arith_shr(self, amount: Self) -> Self {
        // shifting right arithmetically, we need to shift in the sign bit from left
        let sra_shift_fn = |value: concr::Bitvector<L>, amount: concr::Bitvector<L>| {
            if value.is_sign_bit_set() {
                let shifted_mask = concr::Bitvector::<L>::bit_mask().logic_shr(amount);
                Bitwise::bitor(value.logic_shr(amount), shifted_mask.not())
            } else {
                value.logic_shr(amount)
            }
        };

        // the overflow value is determined by sign bit
        let overflow_zeros = if self.is_zeros_sign_bit_set() {
            concr::Bitvector::<L>::bit_mask()
        } else {
            concr::Bitvector::<L>::new(0)
        };

        let overflow_ones = if self.is_ones_sign_bit_set() {
            concr::Bitvector::<L>::bit_mask()
        } else {
            concr::Bitvector::<L>::new(0)
        };
        let overflow_value = Self::from_zeros_ones(overflow_zeros, overflow_ones);

        shift(&self, &amount, sra_shift_fn, sra_shift_fn, &overflow_value)
    }
}

fn shift<const L: u32>(
    value: &ThreeValuedBitvector<L>,
    amount: &ThreeValuedBitvector<L>,
    zeros_shift_fn: impl Fn(concr::Bitvector<L>, concr::Bitvector<L>) -> concr::Bitvector<L>,
    ones_shift_fn: impl Fn(concr::Bitvector<L>, concr::Bitvector<L>) -> concr::Bitvector<L>,
    overflow_value: &ThreeValuedBitvector<L>,
) -> ThreeValuedBitvector<L> {
    if L == 0 {
        // avoid problems with zero-width bitvectors
        return *value;
    }

    let mut zeros = concr::Bitvector::new(0);
    let mut ones = concr::Bitvector::new(0);

    // the shift amount is also three-valued, which poses problems
    // first, if it can be shifted by L or larger value, join by overflow value
    let shift_overflow = amount.umax().as_unsigned() >= L as u64;
    if shift_overflow {
        zeros = zeros.bitor(overflow_value.zeros);
        ones = ones.bitor(overflow_value.ones);
    }

    // only consider the amounts smaller than L afterwards
    let min_shift = amount.umin().as_unsigned().min((L - 1) as u64);
    let max_shift = amount.umax().as_unsigned().min((L - 1) as u64);
    // join by the other shifts iteratively
    for i in min_shift..=max_shift {
        let bi = concr::Bitvector::new(i);
        if amount.contains_concr(&bi) {
            let shifted_zeros = zeros_shift_fn(value.zeros, bi);
            let shifted_ones = ones_shift_fn(value.ones, bi);
            zeros = zeros.bitor(shifted_zeros);
            ones = ones.bitor(shifted_ones);
        }
    }
    ThreeValuedBitvector::from_zeros_ones(zeros, ones)
}
