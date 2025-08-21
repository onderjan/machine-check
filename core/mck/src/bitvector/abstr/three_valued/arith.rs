use super::ThreeValuedBitvector;
use crate::abstr::{PanicBitvector, PanicResult, Phi};
use crate::bitvector::abstr::three_valued::RThreeValuedBitvector;
use crate::bitvector::util;
use crate::concr::{RConcreteBitvector, RSignedBitvector, RUnsignedBitvector};
use crate::forward::HwArith;
use crate::panic::message::{PANIC_NUM_DIV_BY_ZERO, PANIC_NUM_NO_PANIC, PANIC_NUM_REM_BY_ZERO};

impl HwArith for RThreeValuedBitvector {
    type DivRemResult = PanicResult<Self>;

    fn arith_neg(self) -> Self {
        // arithmetic negation
        // since we use wrapping arithmetic, same as subtracting the value from 0
        HwArith::sub(Self::new(0, self.width()), self)
    }
    fn add(self, rhs: Self) -> Self {
        minmax_compute(self, rhs, |lhs, rhs, k| {
            addsub_zeta_k_fn(
                lhs.umin(),
                lhs.umax(),
                rhs.umin(),
                rhs.umax(),
                k,
                |lhs, rhs| lhs.overflowing_add(rhs),
            )
        })
    }
    fn sub(self, rhs: Self) -> Self {
        minmax_compute(self, rhs, |lhs, rhs, k| {
            // swap rhs min and max as it is applied in negative
            addsub_zeta_k_fn(
                lhs.umin(),
                lhs.umax(),
                rhs.umax(),
                rhs.umin(),
                k,
                |lhs, rhs| lhs.overflowing_sub(rhs),
            )
        })
    }
    fn mul(self, rhs: Self) -> Self {
        assert_eq!(self.width(), rhs.width());

        // use the minmax algorithm for now
        minmax_compute(self, rhs, |lhs, rhs, k| {
            // prepare a mask that selects interval [0, k]
            let mod_mask = util::compute_u64_mask(k + 1);

            // convert all to u128 so there is no overflow
            let left_min = (lhs.umin().to_u64() & mod_mask) as u128;
            let right_min = (rhs.umin().to_u64() & mod_mask) as u128;
            let left_max = (lhs.umax().to_u64() & mod_mask) as u128;
            let right_max = (rhs.umax().to_u64() & mod_mask) as u128;

            let zeta_k_min = ((left_min * right_min) >> k) as u64;
            let zeta_k_max = ((left_max * right_max) >> k) as u64;
            (zeta_k_min, zeta_k_max)
        })
    }

    fn udiv(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.width(), rhs.width());
        let width = self.width();

        let min_division_result = (self.umin() / rhs.umax()).result.to_u64();
        let max_division_result = (self.umax() / rhs.umin()).result.to_u64();
        let result = convert_uarith(min_division_result, max_division_result, width);
        panic_result(rhs, result, PANIC_NUM_DIV_BY_ZERO)
    }

    fn sdiv(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.width(), rhs.width());

        let result = compute_sdivrem(self, rhs, |a, b| (a / b).result);
        panic_result(rhs, result, PANIC_NUM_DIV_BY_ZERO)
    }

    fn urem(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.width(), rhs.width());
        let width = self.width();

        let dividend_min = self.umin();
        let dividend_max = self.umax();
        let divisor_min = rhs.umin();
        let divisor_max = rhs.umax();
        let min_division_result = (dividend_min / divisor_max).result.to_u64();
        let max_division_result = (dividend_max / divisor_min).result.to_u64();

        if min_division_result != max_division_result {
            // division results are different, return fully unknown
            let result = RThreeValuedBitvector::new_unknown(width);
            return panic_result(rhs, result, PANIC_NUM_REM_BY_ZERO);
        }

        // division results are the same, return operation result
        let min_result = (dividend_min % divisor_max).result.to_u64();
        let max_result = (dividend_max % divisor_min).result.to_u64();
        let result = convert_uarith(min_result, max_result, width);
        panic_result(rhs, result, PANIC_NUM_REM_BY_ZERO)
    }

    fn srem(self, rhs: Self) -> PanicResult<Self> {
        assert_eq!(self.width(), rhs.width());
        let width = self.width();

        let sdiv_result = self.sdiv(rhs);
        if sdiv_result.result.concrete_value().is_none() {
            // sdiv is not a concrete value, make fully unknown
            let result = RThreeValuedBitvector::new_unknown(width);
            return panic_result(rhs, result, PANIC_NUM_REM_BY_ZERO);
        }

        let result = compute_sdivrem(self, rhs, |a, b| (a % b).result);
        panic_result(rhs, result, PANIC_NUM_REM_BY_ZERO)
    }
}

impl<const W: u32> HwArith for ThreeValuedBitvector<W> {
    type DivRemResult = PanicResult<Self>;

    fn arith_neg(self) -> Self {
        self.to_runtime().arith_neg().unwrap_typed()
    }
    fn add(self, rhs: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.add(rhs).unwrap_typed()
    }
    fn sub(self, rhs: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.sub(rhs).unwrap_typed()
    }
    fn mul(self, rhs: Self) -> Self {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.mul(rhs).unwrap_typed()
    }

    fn udiv(self, rhs: Self) -> PanicResult<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.udiv(rhs).unwrap_typed()
    }

    fn sdiv(self, rhs: Self) -> PanicResult<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.sdiv(rhs).unwrap_typed()
    }

    fn urem(self, rhs: Self) -> PanicResult<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.urem(rhs).unwrap_typed()
    }

    fn srem(self, rhs: Self) -> PanicResult<Self> {
        let (lhs, rhs) = (self.to_runtime(), rhs.to_runtime());
        lhs.srem(rhs).unwrap_typed()
    }
}

fn panic_result(
    divisor: RThreeValuedBitvector,
    result: RThreeValuedBitvector,
    panic_msg_num: u64,
) -> PanicResult<RThreeValuedBitvector> {
    let width = divisor.width();
    let zero = RConcreteBitvector::zero(width);
    let can_panic = divisor.contains_concr(&zero);
    let must_panic = divisor.concrete_value().map(|v| v == zero).unwrap_or(false);
    let panic = if must_panic {
        PanicBitvector::new(panic_msg_num)
    } else if can_panic {
        PanicBitvector::new(PANIC_NUM_NO_PANIC).phi(PanicBitvector::new(panic_msg_num))
    } else {
        PanicBitvector::new(PANIC_NUM_NO_PANIC)
    };
    PanicResult { panic, result }
}

fn minmax_compute(
    lhs: RThreeValuedBitvector,
    rhs: RThreeValuedBitvector,
    zeta_k_fn: fn(RThreeValuedBitvector, RThreeValuedBitvector, u32) -> (u64, u64),
) -> RThreeValuedBitvector {
    let width = lhs.width();
    // from previous paper

    // start with no possibilites
    let mut ones = 0u64;
    let mut zeros = 0u64;

    // iterate over output bits
    for k in 0..width {
        // compute h_k extremes
        let (zeta_k_min, zeta_k_max) = zeta_k_fn(lhs, rhs, k);

        // see if minimum and maximum differs
        if zeta_k_min != zeta_k_max {
            // set result bit unknown
            zeros |= 1 << k;
            ones |= 1 << k;
        } else {
            // set value of bit k, converted to ones-zeros encoding
            zeros |= (!zeta_k_min & 1) << k;
            ones |= (zeta_k_min & 1) << k;
        }
    }
    RThreeValuedBitvector::from_zeros_ones(
        RConcreteBitvector::new(zeros, width),
        RConcreteBitvector::new(ones, width),
    )
}

fn addsub_zeta_k_fn(
    left_min: RUnsignedBitvector,
    left_max: RUnsignedBitvector,
    right_min: RUnsignedBitvector,
    right_max: RUnsignedBitvector,
    k: u32,
    func: fn(u64, u64) -> (u64, bool),
) -> (u64, u64) {
    // prepare a mask that selects interval [0, k]
    let mod_mask = util::compute_u64_mask(k + 1);

    let left_min = left_min.to_u64() & mod_mask;
    let left_max = left_max.to_u64() & mod_mask;
    let right_min = right_min.to_u64() & mod_mask;
    let right_max = right_max.to_u64() & mod_mask;

    // shift right, using the overflow as well
    let zeta_k_min = shr_overflowing(func(left_min, right_min), k);
    let zeta_k_max = shr_overflowing(func(left_max, right_max), k);

    (zeta_k_min, zeta_k_max)
}

fn shr_overflowing(overflowing_result: (u64, bool), k: u32) -> u64 {
    let mut result = overflowing_result.0 >> k;
    if overflowing_result.1 && k > 0 {
        let overflow_pos = u64::BITS - k;
        result |= 1u64 << overflow_pos;
    }
    result
}

fn convert_uarith(min: u64, max: u64, width: u32) -> RThreeValuedBitvector {
    // make highest different bit and all after it unknown
    let different = min ^ max;
    if different == 0 {
        // both are the same
        return RThreeValuedBitvector::new(min, width);
    }

    let highest_different_bit_pos = different.ilog2();
    let unknown_mask = util::compute_u64_mask(highest_different_bit_pos + 1);
    RThreeValuedBitvector::new_value_unknown(
        RConcreteBitvector::new(min, width),
        RConcreteBitvector::new(unknown_mask, width),
    )
}

fn compute_sdivrem(
    dividend: RThreeValuedBitvector,
    divisor: RThreeValuedBitvector,
    op_fn: fn(RSignedBitvector, RSignedBitvector) -> RSignedBitvector,
) -> RThreeValuedBitvector {
    let width = dividend.width();

    if width == 0 {
        // prevent problems
        return dividend;
    }

    let mut zeros = 0u64;
    let mut ones = 0u64;

    let divisor_min = divisor.smin();
    let divisor_max = divisor.smax();
    // handle positive, 0, -1, negative below -1 separately
    if divisor_max.to_i64() > 0 {
        // handle positive divisor
        let divisor_min = if divisor_min.to_i64() > 1 {
            divisor_min
        } else {
            RSignedBitvector::new(1, width)
        };

        apply_signed_op(
            &mut zeros,
            &mut ones,
            dividend.smin(),
            dividend.smax(),
            divisor_min,
            divisor_max,
            op_fn,
        );
    }

    if divisor_min.to_i64() <= 0 && divisor_max.to_i64() >= 0 {
        // 0 divisor, causes division by zero, handle separately

        apply_signed_op(
            &mut zeros,
            &mut ones,
            dividend.smin(),
            dividend.smax(),
            RSignedBitvector::new(0, width),
            RSignedBitvector::new(0, width),
            op_fn,
        );
    }

    if divisor_min.to_i64() <= -1 && divisor_max.to_i64() >= -1 {
        // -1 divisor, causes overflow when the dividend is the most negative value, handle separately
        // handle separately

        let minus_one = dividend.bit_mask_bitvector().cast_signed();

        let mut dividend_min = dividend.smin();
        let dividend_max = dividend.smax();

        if dividend_min == dividend.zeros.sign_bit_mask_bitvector().cast_signed() {
            // overflow
            apply_signed_op(
                &mut zeros,
                &mut ones,
                dividend_min,
                dividend_min,
                minus_one,
                minus_one,
                op_fn,
            );
            if dividend_min != dividend_max {
                dividend_min = dividend_min + RSignedBitvector::new(1, width);
            }
        }

        apply_signed_op(
            &mut zeros,
            &mut ones,
            dividend_min,
            dividend_max,
            minus_one,
            minus_one,
            op_fn,
        );
    }

    if divisor_min.to_i64() < -1 {
        // handle negative divisor
        let divisor_max = if divisor_max.to_i64() < -1 {
            divisor_max
        } else {
            -RSignedBitvector::new(2, width)
        };

        apply_signed_op(
            &mut zeros,
            &mut ones,
            dividend.smin(),
            dividend.smax(),
            divisor_min,
            divisor_max,
            op_fn,
        );
    }

    RThreeValuedBitvector::from_zeros_ones(
        RConcreteBitvector::new(zeros, width),
        RConcreteBitvector::new(ones, width),
    )
}

fn apply_signed_op(
    zeros: &mut u64,
    ones: &mut u64,
    a_min: RSignedBitvector,
    a_max: RSignedBitvector,
    b_min: RSignedBitvector,
    b_max: RSignedBitvector,
    op_fn: fn(RSignedBitvector, RSignedBitvector) -> RSignedBitvector,
) {
    let width = a_min.as_bitvector().width();
    // apply all configurations
    // cast to unsigned u64 afterwards
    let x = op_fn(a_min, b_min).as_bitvector().cast_unsigned().to_u64();
    let y = op_fn(a_min, b_max).as_bitvector().cast_unsigned().to_u64();
    let z = op_fn(a_max, b_min).as_bitvector().cast_unsigned().to_u64();
    let w = op_fn(a_max, b_max).as_bitvector().cast_unsigned().to_u64();

    // find the highest different bit
    let found_zeros = (!x | !y | !z | !w) & util::compute_u64_mask(width);
    let found_ones = x | y | z | w;
    let different = found_zeros & found_ones;

    // apply them
    *zeros |= found_zeros;
    *ones |= found_ones;

    if different == 0 {
        // all are the same
        return;
    }

    // also take care of the lower bits

    let highest_different_bit_pos = different.ilog2();
    let unknown_mask = util::compute_u64_mask(highest_different_bit_pos + 1);

    *zeros |= unknown_mask;
    *ones |= unknown_mask;
}
