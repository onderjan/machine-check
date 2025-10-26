use crate::{
    abstr::{Abstr, BitvectorDomain, PanicBitvector, PanicResult},
    bitvector::interval::SignedInterval,
    concr::ConcreteBitvector,
    forward::HwArith,
    misc::{BitvectorBound, CBound, Join},
    panic::message::{PANIC_NUM_DIV_BY_ZERO, PANIC_NUM_NO_PANIC, PANIC_NUM_REM_BY_ZERO},
};

use super::DualInterval;

impl<B: BitvectorBound> HwArith for DualInterval<B> {
    type DivRemResult = PanicResult<Self>;

    fn arith_neg(self) -> Self {
        // arithmetic negation
        // for wrapping arithmetic, arithmetic negation is same as subtracting the value from 0
        // subtract from interval constructed from 0
        Self::from_value(ConcreteBitvector::zero(self.bound())).sub(self)
    }

    fn add(self, rhs: Self) -> Self {
        Self::resolve_by_wrapping(self, rhs, |a, b| a.hw_add(b))
    }

    fn sub(self, rhs: Self) -> Self {
        Self::resolve_by_wrapping(self, rhs, |a, b| a.hw_sub(b))
    }

    fn mul(self, rhs: Self) -> Self {
        Self::resolve_by_wrapping(self, rhs, |a, b| a.hw_mul(b))
    }

    fn udiv(self, rhs: Self) -> PanicResult<Self> {
        let result = Self::resolve_by_unsigned(self, rhs, |a, b| a.hw_udiv(b));
        let zero = ConcreteBitvector::zero(self.bound());
        let may_panic = rhs.contains_value(&zero);
        let must_panic = rhs.concrete_value() == Some(zero);
        construct_panic_result(result, may_panic, must_panic, PANIC_NUM_DIV_BY_ZERO)
    }

    fn sdiv(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.bound(), rhs.bound());
        let bound = self.bound();
        let mut results = Vec::new();

        let (dividend_near_half, dividend_far_half) = self.opt_halves();
        let (divisor_near_half, divisor_far_half) = rhs.opt_halves();
        if let Some(divisor_near_half) = divisor_near_half {
            let mut divisor_min = divisor_near_half.min();
            let divisor_max = divisor_near_half.max();

            // this can contain zero, in which case the max division result will be all-ones
            if divisor_max.is_zero() {
                // just add the division by zero result which is umax
                results.push(SignedInterval::from_value(
                    ConcreteBitvector::new_umax(bound).as_signed(),
                ));
            } else {
                if divisor_min.is_zero() {
                    // add the division by zero result which is umax
                    results.push(SignedInterval::from_value(
                        ConcreteBitvector::new_umax(bound).as_signed(),
                    ));
                    // increase the min divisor to one
                    divisor_min = ConcreteBitvector::one(bound);
                }

                if let Some(dividend_near_half) = dividend_near_half {
                    let part_near_min =
                        (dividend_near_half.min().as_signed() / divisor_max.as_signed()).result;
                    let part_near_max =
                        (dividend_near_half.max().as_signed() / divisor_min.as_signed()).result;
                    results.push(SignedInterval::new(part_near_min, part_near_max));
                }
                if let Some(dividend_far_half) = dividend_far_half {
                    let part_far_min =
                        (dividend_far_half.min().as_signed() / divisor_min.as_signed()).result;
                    let part_far_max =
                        (dividend_far_half.max().as_signed() / divisor_max.as_signed()).result;

                    results.push(SignedInterval::new(part_far_min, part_far_max));
                }
            }
        }

        if let Some(divisor_far_half) = divisor_far_half {
            // in case the dividend is one followed by zeros (smin, overhalf) and divisor is -1 (all-ones / umax),
            // the division result will overflow to be smin again. This is non-monotone and we have to protect
            // against it.

            fn causes_overflow<B: BitvectorBound>(
                lhs: ConcreteBitvector<B>,
                rhs: ConcreteBitvector<B>,
            ) -> bool {
                lhs == ConcreteBitvector::new_overhalf(lhs.bound())
                    && rhs == ConcreteBitvector::new_umax(rhs.bound())
            }

            let divisor_min = divisor_far_half.min().as_signed();
            let divisor_max = divisor_far_half.max().as_signed();

            // the near half is non-negative, the result will be non-positive
            if let Some(dividend_near_half) = dividend_near_half {
                let part_near_min = (dividend_near_half.max().as_signed() / divisor_max).result;
                let part_near_max = (dividend_near_half.min().as_signed() / divisor_min).result;
                results.push(SignedInterval::new(part_near_min, part_near_max));
            }

            // the far half is negative, the result will be positive
            if let Some(dividend_far_half) = dividend_far_half {
                let far_min_causes_overflow =
                    causes_overflow(dividend_far_half.max(), divisor_min.cast_bitvector());
                let far_max_causes_overflow =
                    causes_overflow(dividend_far_half.min(), divisor_max.cast_bitvector());

                if far_min_causes_overflow && far_max_causes_overflow {
                    // the result is just smin (overhalf)
                    results.push(SignedInterval::from_value(
                        ConcreteBitvector::new_overhalf(bound).as_signed(),
                    ));
                } else {
                    let part_far_min = if far_min_causes_overflow {
                        // use underhalf (smax) instead of overhalf (smin)
                        ConcreteBitvector::new_underhalf(bound).as_signed()
                    } else {
                        (dividend_far_half.max().as_signed() / divisor_min).result
                    };
                    let part_far_max = if far_max_causes_overflow {
                        // add the overflowed result which is just smin (overhalf)
                        results.push(SignedInterval::from_value(
                            ConcreteBitvector::new_overhalf(bound).as_signed(),
                        ));
                        // use underhalf (smax) instead of overhalf (smin)
                        ConcreteBitvector::new_underhalf(bound).as_signed()
                    } else {
                        (dividend_far_half.min().as_signed() / divisor_max).result
                    };

                    results.push(SignedInterval::new(part_far_min, part_far_max));
                }
            }
        }

        let result = DualInterval::from_signed_intervals(&results);

        let zero = ConcreteBitvector::zero(bound);
        let may_panic_zero_division = rhs.contains_value(&zero);
        let must_panic_zero_division = rhs.concrete_value() == Some(zero);

        let may_panic = may_panic_zero_division;
        let must_panic = must_panic_zero_division;

        construct_panic_result(result, may_panic, must_panic, PANIC_NUM_DIV_BY_ZERO)
    }

    fn urem(self, rhs: Self) -> PanicResult<Self> {
        let result = Self::resolve_by_unsigned(self, rhs, |a, b| a.hw_urem(b));
        let zero = ConcreteBitvector::zero(self.bound());
        let may_panic = rhs.contains_value(&zero);
        let must_panic = rhs.concrete_value() == Some(zero);
        construct_panic_result(result, may_panic, must_panic, PANIC_NUM_REM_BY_ZERO)
    }

    fn srem(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.bound(), rhs.bound());
        let bound = self.bound();
        let zero = ConcreteBitvector::zero(bound);
        let may_panic = rhs.contains_value(&zero);
        let must_panic = rhs.concrete_value() == Some(zero);

        // only resolve the remainder values with concrete dividend and divisor
        // which divide to one value
        if let Some(dividend) = self.concrete_value() {
            if let Some(divisor) = rhs.concrete_value() {
                let panic_result = dividend.as_signed() % divisor.as_signed();
                let result = DualInterval::from_value(panic_result.result.cast_bitvector());
                return PanicResult {
                    panic: PanicBitvector::from_concrete(panic_result.panic),
                    result,
                };
            }
        }

        // the remainder must be limited by the divisor
        let divisor_min = rhs.far_half.min().as_signed();
        let divisor_max = rhs.near_half.max().as_signed();

        let zero = ConcreteBitvector::zero(bound).as_signed();
        let one = ConcreteBitvector::one(bound).as_signed();

        let divisor_sign_remainder_min = if divisor_min < zero {
            divisor_min + one
        } else {
            zero
        };

        let divisor_sign_remainder_max = if divisor_max > zero {
            divisor_max - one
        } else {
            zero
        };

        // remainder must have the sign of the dividend instead of the divisor
        // make it double-ended at first and then prune
        let mut remainder_min = divisor_sign_remainder_min.min(-divisor_sign_remainder_max);
        let mut remainder_max = divisor_sign_remainder_max.max(-divisor_sign_remainder_min);

        if self.near_half.is_sign_bit_set() == self.far_half.is_sign_bit_set() {
            if self.near_half.is_sign_bit_set() {
                // only non-positive remainders possible
                remainder_min = remainder_min.min(zero);
                remainder_max = remainder_max.min(zero);
            } else {
                // only non-negative remainders possible
                remainder_min = remainder_min.max(zero);
                remainder_max = remainder_max.max(zero);
            }
        }

        let dividend_min = self.far_half.min().as_signed();
        let dividend_max = self.near_half.max().as_signed();
        if must_panic {
            remainder_min = dividend_min;
            remainder_max = dividend_max;
        } else if may_panic {
            remainder_min = remainder_min.min(dividend_min);
            remainder_max = remainder_max.max(dividend_max);
        }

        let result = DualInterval::from_signed_intervals(&[SignedInterval::new(
            remainder_min,
            remainder_max,
        )]);

        construct_panic_result(result, may_panic, must_panic, PANIC_NUM_REM_BY_ZERO)
    }
}

fn construct_panic_result<T>(
    result: T,
    may_panic: bool,
    must_panic: bool,
    panic_msg_num: u64,
) -> PanicResult<T> {
    let panic = if must_panic {
        PanicBitvector::single_value(panic_msg_num, CBound)
    } else if may_panic {
        PanicBitvector::single_value(PANIC_NUM_NO_PANIC, CBound)
            .join(&PanicBitvector::single_value(panic_msg_num, CBound))
    } else {
        PanicBitvector::single_value(PANIC_NUM_NO_PANIC, CBound)
    };
    PanicResult { panic, result }
}
