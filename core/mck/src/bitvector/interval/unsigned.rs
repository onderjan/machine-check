use std::fmt::Debug;

use crate::{
    bitvector::interval::SignlessInterval,
    concr::{ConcreteBitvector, UnsignedBitvector},
};

/// An unsigned interval with a minimum and a maximum value.
///
/// It is required that min <= max, which means the interval
/// does not support wrapping nor representing an empty set.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct UnsignedInterval<const W: u32> {
    pub(super) min: UnsignedBitvector<W>,
    pub(super) max: UnsignedBitvector<W>,
}

impl<const W: u32> UnsignedInterval<W> {
    pub const FULL: Self = Self {
        min: ConcreteBitvector::<W>::zero().cast_unsigned(),
        max: ConcreteBitvector::<W>::const_umax().cast_unsigned(),
    };

    pub fn new(min: UnsignedBitvector<W>, max: UnsignedBitvector<W>) -> Self {
        assert!(min <= max);
        Self { min, max }
    }

    pub fn contains_value(&self, value: UnsignedBitvector<W>) -> bool {
        self.min <= value && value <= self.max
    }

    pub fn min(&self) -> UnsignedBitvector<W> {
        self.min
    }
    pub fn max(&self) -> UnsignedBitvector<W> {
        self.max
    }

    pub fn hw_udiv(self, rhs: Self) -> Self {
        // division is monotone wrt. dividend and anti-monotone wrt. divisor
        let result_min = (self.min / rhs.max).result;
        let result_max = (self.max / rhs.min).result;
        Self {
            min: result_min,
            max: result_max,
        }
    }

    pub fn hw_urem(self, rhs: Self) -> Self {
        let div_result = self.hw_udiv(rhs);
        if div_result.min != div_result.max {
            // division is not a concrete value
            // estimate that the maximum remainder is equal to the maximum divisor minus 1
            // if division by zero is possible, the remainder can be the dividend
            // so allow it in the estimate
            let zero = ConcreteBitvector::zero().cast_unsigned();
            let max_candidate_from_divisor = if rhs.max.is_nonzero() {
                rhs.max - ConcreteBitvector::one().cast_unsigned()
            } else {
                zero
            };
            let max_candidate_from_dividend = if rhs.min.is_nonzero() { zero } else { self.max };

            return Self {
                min: ConcreteBitvector::zero().cast_unsigned(),
                max: max_candidate_from_divisor.max(max_candidate_from_dividend),
            };
        }

        // division results are the same, return remainder bounds
        let remainder_min = self.min % rhs.max;
        let remainder_max = self.max % rhs.min;
        Self {
            min: remainder_min.result,
            max: remainder_max.result,
        }
    }

    pub fn ext<const X: u32>(self) -> UnsignedInterval<X> {
        if self.min == self.max {
            // clearly, we can extend
            let ext_value = self.min.ext();
            return UnsignedInterval {
                min: ext_value,
                max: ext_value,
            };
        }

        // if we narrow the interval and disregarded a bound, saturate
        let mut ext_min: UnsignedBitvector<X> = self.min.ext();
        let mut ext_max: UnsignedBitvector<X> = self.max.ext();

        let min_diff = self.min - ext_min.ext();
        let max_diff = self.max - ext_max.ext();

        if min_diff != max_diff {
            // we disregarded a bound, saturate
            ext_min = ConcreteBitvector::zero().cast_unsigned();
            ext_max = ConcreteBitvector::const_umax().cast_unsigned();
        }
        UnsignedInterval {
            min: ext_min,
            max: ext_max,
        }
    }

    pub fn try_into_signless(self) -> Option<SignlessInterval<W>> {
        if self.min.as_bitvector().is_sign_bit_set() == self.max.as_bitvector().is_sign_bit_set() {
            Some(SignlessInterval {
                min: self.min.as_bitvector(),
                max: self.max.as_bitvector(),
            })
        } else {
            None
        }
    }

    pub fn bit_and(self, rhs: Self) -> Self {
        // An improvement of the Hacker's Delight algorithm, giving O(1) computation
        // if Count Leading Zeros (clz) is implemented.

        let (x_p, x_q) = (self.min.to_u64(), self.max.to_u64());
        let (y_p, y_q) = (rhs.min.to_u64(), rhs.max.to_u64());

        let x_diff_mask = mask_from_leading_one(x_p ^ x_q);
        let y_diff_mask = mask_from_leading_one(y_p ^ y_q);
        let diff_mask = x_diff_mask | y_diff_mask;

        let min = x_p & y_p & !mask_from_leading_one(!x_p & !y_p & diff_mask);
        let max = {
            let selection_x = mask_from_leading_one(x_q & !y_q & x_diff_mask);
            let selection_y = mask_from_leading_one(y_q & !x_q & y_diff_mask);

            let result_q = x_q & y_q;
            let result_x = selection_x & (y_q & !x_q);
            let result_y = selection_y & (x_q & !y_q);
            result_q | result_x.max(result_y)
        };

        Self::new(
            ConcreteBitvector::new(min).cast_unsigned(),
            ConcreteBitvector::new(max).cast_unsigned(),
        )
    }

    pub fn bit_or(self, rhs: Self) -> Self {
        // An improvement of the Hacker's Delight algorithm, giving O(1) computation
        // if Count Leading Zeros (clz) is implemented.

        let (x_p, x_q) = (self.min.to_u64(), self.max.to_u64());
        let (y_p, y_q) = (rhs.min.to_u64(), rhs.max.to_u64());

        let x_diff_mask = mask_from_leading_one(x_p ^ x_q);
        let y_diff_mask = mask_from_leading_one(y_p ^ y_q);
        let diff_mask = x_diff_mask | y_diff_mask;

        let min = {
            let candidates_x = y_p & !x_p & x_diff_mask;
            let candidates_y = x_p & !y_p & y_diff_mask;

            if candidates_x >= candidates_y {
                let selection_x = mask_from_leading_one(candidates_x);
                (x_p & !selection_x) | y_p
            } else {
                let selection_y = mask_from_leading_one(candidates_y);
                (y_p & !selection_y) | x_p
            }
        };
        let max = x_q | y_q | mask_from_leading_one(x_q & y_q & diff_mask);

        Self::new(
            ConcreteBitvector::new(min).cast_unsigned(),
            ConcreteBitvector::new(max).cast_unsigned(),
        )
    }

    pub fn bit_xor(self, rhs: Self) -> Self {
        // An improvement of the Hacker's Delight algorithm, giving O(1) computation
        // if Count Leading Zeros (clz) is implemented.

        let (x_p, x_q) = (self.min.to_u64(), self.max.to_u64());
        let (y_p, y_q) = (rhs.min.to_u64(), rhs.max.to_u64());

        let diff_mask = mask_from_leading_one((x_p ^ x_q) | (y_p ^ y_q));

        let min = {
            let y_q_mask = mask_from_leading_one(!x_p & y_q & diff_mask);
            let x_q_mask = mask_from_leading_one(!y_p & x_q & diff_mask);
            (x_p & !y_q & !y_q_mask) | (y_p & !x_q & !x_q_mask)
        };

        let max = {
            let neither_p_mask = mask_from_leading_one(!x_p & !y_p & diff_mask);
            let both_q_mask = mask_from_leading_one(y_q & x_q & diff_mask);
            (!x_p | !y_p | neither_p_mask) & (x_q | y_q | both_q_mask)
        };

        // we need to mask max
        let max = max & ConcreteBitvector::<W>::bit_mask().as_unsigned();

        Self::new(
            ConcreteBitvector::new(min).cast_unsigned(),
            ConcreteBitvector::new(max).cast_unsigned(),
        )
    }
}

fn mask_from_leading_one(x: u64) -> u64 {
    let diff_clz = x.leading_zeros();
    u64::MAX.checked_shr(diff_clz).unwrap_or(0)
}

impl<const W: u32> Debug for UnsignedInterval<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.min, self.max)
    }
}
