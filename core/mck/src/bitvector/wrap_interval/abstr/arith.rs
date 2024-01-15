use super::Bitvector;
use crate::{
    bitvector::{concrete::ConcreteBitvector, wrap_interval::interval::Interval},
    forward::{HwArith, TypedCmp},
    types::Unsigned,
};

impl<const L: u32> HwArith for Bitvector<L> {
    fn arith_neg(self) -> Self {
        // arithmetic negation
        // since we use wrapping arithmetic, same as subtracting the value from 0
        HwArith::sub(Self::new(0), self)
    }

    fn add(self, rhs: Self) -> Self {
        // ensure the produced bounds are less than 2^L apart, produce a full interval otherwise
        if self.addsub_full_override(rhs) {
            Self::full()
        } else {
            // wrapping and fully monotonic: add bounds
            let start = self.start.add(rhs.start);
            let end = self.end.add(rhs.end);

            Self { start, end }
        }
    }

    fn sub(self, rhs: Self) -> Self {
        // ensure the produced bounds are less than 2^L apart, produce a full interval otherwise
        if self.addsub_full_override(rhs) {
            Self::full()
        } else {
            // wrapping, monotonic on lhs, anti-monotonic on rhs: subtract bounds, remember to flip rhs bounds
            let start = self.start.sub(rhs.end);
            let end = self.end.sub(rhs.start);

            Self { start, end }
        }
    }

    fn mul(self, rhs: Self) -> Self {
        // TODO: make multiplication work correctly
        if L == 0 {
            // concrete bitvector const one cannot be added here, prevent assert
            return self;
        }

        let lhs_start = self.start;
        let rhs_start = rhs.start;
        let start = lhs_start.mul(rhs_start);

        let lhs_diff = self.bound_diff();

        let rhs_diff = rhs.bound_diff();

        //println!("{} * {}: bound_diff {}, {}", self, rhs, lhs_diff, rhs_diff);

        let Some(diff_product) = lhs_diff.checked_mul(rhs_diff) else {
            return Self::full();
        };
        let Some(diff_start_product) = lhs_diff.checked_mul(rhs_start) else {
            return Self::full();
        };
        let Some(start_diff_product) = lhs_start.checked_mul(rhs_diff) else {
            return Self::full();
        };

        /*println!(
            "Products: {} + {} + {}",
            diff_product, diff_start_product, start_diff_product
        );*/

        let Some(result_len) = diff_product.checked_add(diff_start_product).and_then(|v| v.checked_add(start_diff_product)) else {
            return Self::full();
        };

        let end = start.add(result_len);

        Self { start, end }
    }

    fn udiv(self, rhs: Self) -> Self {
        if L == 0 {
            return self;
        }
        let lhs_intervals = self.unsigned_intervals();
        let mut rhs_intervals = rhs.unsigned_intervals();

        if let Some(rhs_first) = rhs_intervals.first() {
            if rhs_first.min == Unsigned::zero() {
                let mut new_rhs_intervals = vec![Interval::new(Unsigned::zero(), Unsigned::zero())];
                if rhs_first.max != Unsigned::zero() {
                    new_rhs_intervals.push(Interval::new(Unsigned::one(), rhs_first.max));
                }
                new_rhs_intervals.extend(rhs_intervals.iter().skip(1));
                rhs_intervals = new_rhs_intervals;
            }
        }
        let mut result_intervals = Vec::new();

        for lhs_interval in lhs_intervals.iter() {
            for rhs_interval in rhs_intervals.iter() {
                let result_max = lhs_interval.max / rhs_interval.min;
                let result_min = lhs_interval.min / rhs_interval.max;
                result_intervals.push(Interval::new(result_min, result_max));
            }
        }

        /*println!(
            "{:?} / {:?} = {:?}",
            lhs_intervals, rhs_intervals, result_intervals
        );*/

        Self::from_unsigned_intervals(result_intervals)
    }

    fn sdiv(self, rhs: Self) -> Self {
        // resolve by parts, division by zero returns all ones
        // sign of result is positive if signs of both operands are the same
        self.sdivrem_by_parts(
            rhs,
            |a, b| a.udiv(b),
            |a, b| a == b,
            Self::from_concrete(Self::representable_umax()),
        )
    }

    fn urem(self, rhs: Self) -> Self {
        if L == 0 {
            return self;
        }
        let Some(rhs_value) = rhs.concrete_value() else {
            // multiple divisor values, assume minimal divisor is nonzero first
            let rhs_max = rhs.umax();
            let lhs_min = if self.umax().as_unsigned() < rhs.umin().as_unsigned() {
                self.umin()
            } else {
                ConcreteBitvector::new(0)
            };
            let mut result_min = lhs_min.as_unsigned().min(rhs_max.as_unsigned());
            let mut result_max = rhs_max.as_unsigned();
            // adjust for zero divisor: in that case, dividend is returned, so adjust the minimum and maximum accordingly
            if rhs.contains_concrete(&ConcreteBitvector::new(0)) {
                result_min = result_min.min(lhs_min.as_unsigned());
                let lhs_max = self.umax();
                result_max = result_max.max(lhs_max.as_unsigned());
            }

            return Self::from_wrap_interval(ConcreteBitvector::new(result_min), ConcreteBitvector::new(result_max));
        };

        if rhs_value.is_zero() {
            // in case of zero divisor, the dividend is returned
            return self;
        }

        // single divisor value, decide if the result is definitely full
        let lhs_diff = self.bound_diff();
        if rhs_value.typed_ulte(lhs_diff).is_nonzero() {
            return Self::full();
        }

        // decide if the result wraps ... if it does, return full
        if self.start.as_unsigned() > self.end.as_unsigned() {
            return Self::full();
        }

        // return remainder
        Self::from_wrap_interval(self.start.urem(rhs_value), self.end.urem(rhs_value))
    }

    fn srem(self, rhs: Self) -> Self {
        // resolve by parts, remainder of division by zero is the dividend
        // sign of the result equals sign of the dividend
        self.sdivrem_by_parts(rhs, |a, b| a.urem(b), |a, _b| a, self)
    }
}

impl<const L: u32> Bitvector<L> {
    fn addsub_full_override(self, rhs: Self) -> bool {
        let lhs_diff = self.bound_diff();
        let rhs_diff = rhs.bound_diff();

        let wrapped_total_len = lhs_diff.add(rhs_diff);
        wrapped_total_len.as_unsigned() < lhs_diff.as_unsigned()
            || wrapped_total_len.as_unsigned() < rhs_diff.as_unsigned()
    }

    fn sdivrem_by_parts(
        self,
        rhs: Self,
        op: fn(Bitvector<L>, Bitvector<L>) -> Bitvector<L>,
        sign_op: fn(bool, bool) -> bool,
        zero_second_result: Bitvector<L>,
    ) -> Bitvector<L> {
        let mut interval_results = Vec::new();

        let lhs_abs_neg_intervals = self.absolute_negative_intervals();
        let lhs_nonneg_intervals = self.nonnegative_intervals();
        let rhs_abs_neg_intervals = rhs.absolute_negative_intervals();
        let rhs_pos_intervals = rhs.positive_intervals();

        // resolve for intervals of both
        Self::sdivrem_part(
            &mut interval_results,
            op,
            sign_op(false, false),
            &lhs_abs_neg_intervals,
            &rhs_abs_neg_intervals,
        );
        Self::sdivrem_part(
            &mut interval_results,
            op,
            sign_op(true, false),
            &lhs_nonneg_intervals,
            &rhs_abs_neg_intervals,
        );
        Self::sdivrem_part(
            &mut interval_results,
            op,
            sign_op(false, true),
            &lhs_abs_neg_intervals,
            &rhs_pos_intervals,
        );
        Self::sdivrem_part(
            &mut interval_results,
            op,
            sign_op(true, true),
            &lhs_nonneg_intervals,
            &rhs_pos_intervals,
        );

        // resolve division by zero if necessary
        if rhs.contains_concrete(&ConcreteBitvector::new(0)) {
            interval_results.push(zero_second_result);
        }

        Bitvector::<L>::from_unsigned_intervals(
            interval_results
                .iter()
                .flat_map(|v| v.unsigned_intervals())
                .collect(),
        )
    }

    fn sdivrem_part(
        interval_results: &mut Vec<Bitvector<L>>,
        op: fn(Bitvector<L>, Bitvector<L>) -> Bitvector<L>,
        positive_sign: bool,
        lhs_intervals: &[Interval<Unsigned<L>>],
        rhs_intervals: &[Interval<Unsigned<L>>],
    ) {
        for lhs_interval in lhs_intervals.iter().cloned() {
            for rhs_interval in rhs_intervals.iter().cloned() {
                let mut op_result = op(
                    Bitvector::<L>::from_unsigned_interval(lhs_interval),
                    Bitvector::<L>::from_unsigned_interval(rhs_interval),
                );
                if !positive_sign {
                    op_result = op_result.arith_neg()
                }
                interval_results.push(op_result);
            }
        }
    }
}
