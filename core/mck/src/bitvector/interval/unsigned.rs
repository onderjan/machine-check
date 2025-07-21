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

        // minimum with explicit operands
        /*let min = {
            let selection_x = mask_from_leading_one(!x_p & !y_p & x_diff_mask);
            let selection_y = mask_from_leading_one(!x_p & !y_p & y_diff_mask);

            //let result_x = (x_p & !selection_x) & y_p;
            let sensitive_x = selection_x & !(selection_x >> 1);
            let operand_x = (x_p & (!selection_x >> 1)) | sensitive_x;
            assert!(x_p <= operand_x && operand_x <= x_q);
            let result_x = operand_x & y_p;

            //let result_y = x_p & (y_p & !selection_y);

            let sensitive_y = selection_y & !(selection_y >> 1);
            let operand_y = (y_p & (!selection_y >> 1)) | sensitive_y;
            assert!(y_p <= operand_y && operand_y <= y_q);
            let result_y = x_p & operand_y;

            result_x.min(result_y)
        };*/

        let max = {
            let selection_x = mask_from_leading_one(x_q & !y_q & x_diff_mask);
            let selection_y = mask_from_leading_one(y_q & !x_q & y_diff_mask);

            let result_x = (x_q | selection_x) & y_q;

            /*let sensitive_x = selection_x & !(selection_x >> 1);
            let operand_x = (x_q | (selection_x >> 1)) & !sensitive_x;
            assert!(x_p <= operand_x && operand_x <= x_q);
            let result_x = operand_x & y_q;*/

            let result_y = (x_q) & (y_q | selection_y);

            /*let sensitive_y = selection_y & !(selection_y >> 1);
            let operand_y = (y_q | (selection_y >> 1)) & !sensitive_y;
            assert!(y_p <= operand_y && operand_y <= y_q);
            let result_y = x_q & operand_y;*/

            result_x.max(result_y)
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
            let selection_x = mask_from_leading_one(y_p & !x_p & x_diff_mask);
            let selection_y = mask_from_leading_one(x_p & !y_p & y_diff_mask);

            let result_x = (x_p & !selection_x) | y_p;

            /*let sensitive_x = selection_x & !(selection_x >> 1);
            let operand_x = (x_p & !(selection_x >> 1)) | sensitive_x;
            assert!(x_p <= operand_x && operand_x <= x_q);
            let result_x = operand_x | y_p;*/

            let result_y = (y_p & !selection_y) | x_p;

            /*let sensitive_y = selection_y & !(selection_y >> 1);
            let operand_y = (y_p & !(selection_y >> 1)) | sensitive_y;
            assert!(y_p <= operand_y && operand_y <= y_q);
            let result_y = x_p | operand_y;*/

            result_x.min(result_y)
        };

        let max = x_q | y_q | mask_from_leading_one(x_q & y_q & diff_mask);

        // maximum with explicit operands
        /*let max = {
            let selection_x = mask_from_leading_one(x_q & y_q & x_diff_mask);
            let selection_y = mask_from_leading_one(x_q & y_q & y_diff_mask);

            //let result_x = (x_q | selection_x) | y_q;
            let sensitive_x = selection_x & !(selection_x >> 1);
            let operand_x = (x_q | (selection_x >> 1)) & !sensitive_x;
            assert!(x_p <= operand_x && operand_x <= x_q);
            let result_x = operand_x | y_q;

            //let result_y = x_q | (y_q | selection_y);

            let sensitive_y = selection_y & !(selection_y >> 1);
            let operand_y = (y_q | (selection_y >> 1)) & !sensitive_y;
            assert!(y_p <= operand_y && operand_y <= y_q);
            let result_y = x_q | operand_y;

            result_x.max(result_y)
        };*/

        Self::new(
            ConcreteBitvector::new(min).cast_unsigned(),
            ConcreteBitvector::new(max).cast_unsigned(),
        )
    }

    pub fn bit_xor(self, rhs: Self) -> Self {
        // An improvement of the Hacker's Delight algorithm, giving O(1) computation
        // if Count Leading Zeros (clz) is implemented.

        // make sure a diff mask is greater or equal to b diff mask
        let (a_p, a_q, b_p, b_q) = {
            let (x_p, x_q) = (self.min.to_u64(), self.max.to_u64());
            let (y_p, y_q) = (rhs.min.to_u64(), rhs.max.to_u64());

            if x_p ^ x_q >= y_p ^ y_q {
                (x_p, x_q, y_p, y_q)
            } else {
                (y_p, y_q, x_p, x_q)
            }
        };

        let a_diff_mask = mask_from_leading_one(a_p ^ a_q);

        let min = {
            let b_q_mask = mask_from_leading_one(!a_p & b_q & a_diff_mask);
            let a_q_mask = mask_from_leading_one(!b_p & a_q & a_diff_mask);

            (a_p & !b_q & !b_q_mask) | (b_p & !a_q & !a_q_mask)
        };

        let neither_p_mask = mask_from_leading_one(!a_p & !b_p & a_diff_mask);
        let both_q_mask = mask_from_leading_one(a_q & b_q & a_diff_mask);

        // maximum with explicit operands
        /*let max = {
            let b_diff_mask = mask_from_leading_one(b_p ^ b_q);
            if a_diff_mask > b_diff_mask {
                let leading_a = lead_mask(a_diff_mask);

                assert_eq!(b_p & leading_a, b_q & leading_a);
                if (b_q & leading_a) != 0 {
                    let operand_x = (a_p & !neither_p_mask) | (!b_p & neither_p_mask);
                    assert!(a_p <= operand_x && operand_x <= a_q);
                    operand_x ^ b_p
                    //(a_p ^ b_p) | neither_p_mask
                } else {
                    let operand_x = (a_q & !both_q_mask) | (!b_q & both_q_mask);
                    assert!(a_p <= operand_x && operand_x <= a_q);
                    operand_x ^ b_q
                    //(a_q ^ b_q) | both_q_mask
                }
            } else {
                let diff_mask_lead = lead_mask(a_diff_mask);

                let operand_x = (a_p | a_diff_mask) & !diff_mask_lead;
                let operand_y = (b_q & !a_diff_mask) | diff_mask_lead;

                operand_x ^ operand_y
                //(a_p ^ b_p) | a_diff_mask
            }
        };*/

        let max = (!a_p | !b_p | neither_p_mask) & (a_q | b_q | both_q_mask);

        // we need to mask max
        let max = max & ConcreteBitvector::<W>::bit_mask().to_u64();

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

#[allow(dead_code)]
fn lead_mask(x: u64) -> u64 {
    if let Some(ilog2) = x.checked_ilog2() {
        1u64 << ilog2
    } else {
        0
    }
}

impl<const W: u32> Debug for UnsignedInterval<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.min, self.max)
    }
}
