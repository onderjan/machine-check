use crate::{
    bitvector::{concrete::ConcreteBitvector, wrap_interval::interval::Interval},
    forward::{HwArith, HwShift},
};

use super::Bitvector;

impl<const L: u32> HwShift for Bitvector<L> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        // shifting left logically
        let mut intervals = Vec::new();

        for amount_interval in amount.unsigned_intervals() {
            // adjust too large amounts first
            let amount_interval = if amount_interval.max >= L as u64 {
                // amount too large, add zero interval
                intervals.push(Interval::new(0, 0));
                if amount_interval.min >= L as u64 {
                    // any value from the amount interval will result in zero
                    continue;
                }
                Interval::new(amount_interval.min, (L - 1) as u64)
            } else {
                amount_interval
            };

            for amount in amount_interval.min..=amount_interval.max {
                // we will multiply bound diff by 2^amount and see if this results in interval overflow
                let multiplier = ConcreteBitvector::new(1u64 << amount);
                if self.bound_diff().checked_mul(multiplier).is_some() {
                    // not full
                    let start = self.start.logic_shl(ConcreteBitvector::new(amount));
                    let end = self.end.logic_shl(ConcreteBitvector::new(amount));
                    let result = Bitvector::from_wrap_interval(start, end);
                    intervals.extend(result.unsigned_intervals());
                } else {
                    // can be full
                    return Self::full();
                }
            }
        }

        Self::from_intervals(intervals)
    }

    fn logic_shr(self, amount: Self) -> Self {
        // shifting right logically
        let mut intervals = Vec::new();

        for amount_interval in amount.unsigned_intervals() {
            // adjust too large amounts first
            let amount_interval = if amount_interval.max >= L as u64 {
                // amount too large, add zero interval
                intervals.push(Interval::new(0, 0));
                if amount_interval.min >= L as u64 {
                    // any value from the amount interval will result in zero
                    continue;
                }
                Interval::new(amount_interval.min, (L - 1) as u64)
            } else {
                amount_interval
            };

            for amount in amount_interval.min..=amount_interval.max {
                // look at all unsigned intervals
                for lhs_interval in self.unsigned_intervals() {
                    // no interval overflow possible, just shift right
                    let min = ConcreteBitvector::<L>::new(lhs_interval.min)
                        .logic_shr(ConcreteBitvector::new(amount));
                    let max = ConcreteBitvector::<L>::new(lhs_interval.max)
                        .logic_shr(ConcreteBitvector::new(amount));
                    intervals.push(Interval::new(min.as_unsigned(), max.as_unsigned()));
                }
            }
        }

        Self::from_intervals(intervals)
    }

    fn arith_shr(self, amount: Self) -> Self {
        if L == 0 {
            return self;
        }
        // shifting right arithmetically
        let mut intervals = Vec::new();

        for amount_interval in amount.unsigned_intervals() {
            // adjust too large amounts first
            let amount_interval = if amount_interval.max >= L as u64 {
                // amount too large, add intervals that copy possible result signs
                if self.intersects(&Bitvector::from_wrap_interval(
                    Self::representable_smin(),
                    Self::representable_umax(),
                )) {
                    let all_ones = ConcreteBitvector::<L>::new(1).arith_neg().as_unsigned();
                    intervals.push(Interval::new(all_ones, all_ones));
                }
                if self.intersects(&Bitvector::from_wrap_interval(
                    Self::representable_umin(),
                    Self::representable_smax(),
                )) {
                    intervals.push(Interval::new(0, 0));
                }
                if amount_interval.min >= L as u64 {
                    // any value from the amount interval is too large
                    continue;
                }
                Interval::new(amount_interval.min, (L - 1) as u64)
            } else {
                amount_interval
            };

            for amount in amount_interval.min..=amount_interval.max {
                // look at negative intervals first
                for lhs_interval in self.negative_intervals() {
                    // no interval overflow possible, just shift right
                    // perform arithmetic negation afterwards to convert absolute negative to unsigned
                    let min = ConcreteBitvector::<L>::new(lhs_interval.min)
                        .arith_shr(ConcreteBitvector::new(amount));
                    let max = ConcreteBitvector::<L>::new(lhs_interval.max)
                        .arith_shr(ConcreteBitvector::new(amount));
                    // it is possible that there will be wrap to zero
                    intervals.extend(Bitvector::from_wrap_interval(min, max).unsigned_intervals());
                }
                // look at nonnegative intervals next
                for lhs_interval in self.nonnegative_intervals() {
                    // no interval overflow possible, just shift right
                    let min = ConcreteBitvector::<L>::new(lhs_interval.min)
                        .logic_shr(ConcreteBitvector::new(amount));
                    let max = ConcreteBitvector::<L>::new(lhs_interval.max)
                        .logic_shr(ConcreteBitvector::new(amount));
                    intervals.push(Interval::new(min.as_unsigned(), max.as_unsigned()));
                }
            }
        }

        Self::from_intervals(intervals)
    }
}
