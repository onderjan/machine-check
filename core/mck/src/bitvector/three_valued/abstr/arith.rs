use super::ThreeValuedBitvector;
use crate::bitvector::concrete::ConcreteBitvector;
use crate::bitvector::util;
use crate::forward::HwArith;

impl<const L: u32> HwArith for ThreeValuedBitvector<L> {
    fn neg(self) -> Self {
        // arithmetic negation
        // since we use wrapping arithmetic, same as subtracting the value from 0
        HwArith::sub(Self::new(0), self)
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
        // use the minmax algorithm for now
        minmax_compute(self, rhs, |lhs, rhs, k| {
            // prepare a mask that selects interval [0, k]
            let mod_mask = util::compute_u64_mask(k + 1);

            // convert all to u128 so there is no overflow
            let left_min = (lhs.umin().as_unsigned() & mod_mask) as u128;
            let right_min = (rhs.umin().as_unsigned() & mod_mask) as u128;
            let left_max = (lhs.umax().as_unsigned() & mod_mask) as u128;
            let right_max = (rhs.umax().as_unsigned() & mod_mask) as u128;

            let zeta_k_min = ((left_min * right_min) >> k) as u64;
            let zeta_k_max = ((left_max * right_max) >> k) as u64;
            (zeta_k_min, zeta_k_max)
        })
    }

    fn udiv(self, rhs: Self) -> Self {
        let min_division_result = self.umin().udiv(rhs.umax()).as_unsigned();
        let max_division_result = self.umax().udiv(rhs.umin()).as_unsigned();
        convert_uarith(min_division_result, max_division_result)
    }

    fn sdiv(self, rhs: Self) -> Self {
        compute_sdivrem(self, rhs, |a, b| a.sdiv(b))
    }

    fn urem(self, rhs: Self) -> Self {
        let dividend_min = self.umin();
        let dividend_max = self.umax();
        let divisor_min = rhs.umin();
        let divisor_max = rhs.umax();
        let min_division_result = dividend_min.udiv(divisor_max).as_unsigned();
        let max_division_result = dividend_max.udiv(divisor_min).as_unsigned();

        if min_division_result != max_division_result {
            // division results are different, return fully unknown
            return ThreeValuedBitvector::new_unknown();
        }

        // division results are the same, return operation result
        let min_result = dividend_min.urem(divisor_max).as_unsigned();
        let max_result = dividend_max.urem(divisor_min).as_unsigned();
        convert_uarith(min_result, max_result)
    }

    fn srem(self, rhs: Self) -> Self {
        let sdiv_result = self.sdiv(rhs);
        if sdiv_result.concrete_value().is_none() {
            // sdiv is not a concrete value, make fully unknown
            return Self::new_unknown();
        }

        compute_sdivrem(self, rhs, |a, b| a.srem(b))
    }
}

fn minmax_compute<const L: u32>(
    lhs: ThreeValuedBitvector<L>,
    rhs: ThreeValuedBitvector<L>,
    zeta_k_fn: fn(ThreeValuedBitvector<L>, ThreeValuedBitvector<L>, u32) -> (u64, u64),
) -> ThreeValuedBitvector<L> {
    // from previous paper

    // start with no possibilites
    let mut ones = 0u64;
    let mut zeros = 0u64;

    // iterate over output bits
    for k in 0..L {
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
    ThreeValuedBitvector::from_zeros_ones(
        ConcreteBitvector::new(zeros),
        ConcreteBitvector::new(ones),
    )
}

fn addsub_zeta_k_fn<const L: u32>(
    left_min: ConcreteBitvector<L>,
    left_max: ConcreteBitvector<L>,
    right_min: ConcreteBitvector<L>,
    right_max: ConcreteBitvector<L>,
    k: u32,
    func: fn(u64, u64) -> (u64, bool),
) -> (u64, u64) {
    // prepare a mask that selects interval [0, k]
    let mod_mask = util::compute_u64_mask(k + 1);

    let left_min = left_min.as_unsigned() & mod_mask;
    let left_max = left_max.as_unsigned() & mod_mask;
    let right_min = right_min.as_unsigned() & mod_mask;
    let right_max = right_max.as_unsigned() & mod_mask;

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

fn convert_uarith<const L: u32>(min: u64, max: u64) -> ThreeValuedBitvector<L> {
    // make highest different bit and all after it unknown
    let different = min ^ max;
    if different == 0 {
        // both are the same
        return ThreeValuedBitvector::new(min);
    }

    let highest_different_bit_pos = different.ilog2();
    let unknown_mask = util::compute_u64_mask(highest_different_bit_pos + 1);
    ThreeValuedBitvector::new_value_unknown(
        ConcreteBitvector::new(min),
        ConcreteBitvector::new(unknown_mask),
    )
}

fn compute_sdivrem<const L: u32>(
    dividend: ThreeValuedBitvector<L>,
    divisor: ThreeValuedBitvector<L>,
    op_fn: fn(ConcreteBitvector<L>, ConcreteBitvector<L>) -> ConcreteBitvector<L>,
) -> ThreeValuedBitvector<L> {
    if L == 0 {
        // prevent problems
        return dividend;
    }

    let mut zeros = 0u64;
    let mut ones = 0u64;

    let divisor_min = divisor.smin();
    let divisor_max = divisor.smax();
    // handle positive, 0, -1, negative below -1 separately
    if divisor_max.as_signed() > 0 {
        // handle positive divisor
        let divisor_min = if divisor_min.as_signed() > 1 {
            divisor_min
        } else {
            ConcreteBitvector::new(1)
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

    if divisor_min.as_signed() <= 0 && divisor_max.as_signed() >= 0 {
        // 0 divisor, causes division by zero, handle separately

        apply_signed_op(
            &mut zeros,
            &mut ones,
            dividend.smin(),
            dividend.smax(),
            ConcreteBitvector::new(0),
            ConcreteBitvector::new(0),
            op_fn,
        );
    }

    if divisor_min.as_signed() <= -1 && divisor_max.as_signed() >= -1 {
        // -1 divisor, causes overflow when the dividend is the most negative value, handle separately
        // handle separately

        let minus_one = ConcreteBitvector::bit_mask();

        let mut dividend_min = dividend.smin();
        let dividend_max = dividend.smax();

        if dividend_min == ConcreteBitvector::sign_bit_mask() {
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
                dividend_min = dividend_min.add(ConcreteBitvector::new(1));
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

    if divisor_min.as_signed() < -1 {
        // handle negative divisor
        let divisor_max = if divisor_max.as_signed() < -1 {
            divisor_max
        } else {
            ConcreteBitvector::new(2).neg()
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

    ThreeValuedBitvector::from_zeros_ones(
        ConcreteBitvector::new(zeros),
        ConcreteBitvector::new(ones),
    )
}

fn apply_signed_op<const L: u32>(
    zeros: &mut u64,
    ones: &mut u64,
    a_min: ConcreteBitvector<L>,
    a_max: ConcreteBitvector<L>,
    b_min: ConcreteBitvector<L>,
    b_max: ConcreteBitvector<L>,
    op_fn: fn(ConcreteBitvector<L>, ConcreteBitvector<L>) -> ConcreteBitvector<L>,
) {
    // apply all configurations
    let x = op_fn(a_min, b_min).as_unsigned();
    let y = op_fn(a_min, b_max).as_unsigned();
    let z = op_fn(a_max, b_min).as_unsigned();
    let w = op_fn(a_max, b_max).as_unsigned();

    // find the highest different bit
    let found_zeros = (!x | !y | !z | !w) & util::compute_u64_mask(L);
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
