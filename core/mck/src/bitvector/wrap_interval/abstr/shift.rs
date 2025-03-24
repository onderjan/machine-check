use crate::{
    bitvector::support::UnsignedBitvector,
    bitvector::{concrete::ConcreteBitvector, wrap_interval::interval::Interval},
    forward::{HwArith, HwShift},
};

use super::Bitvector;

impl<const L: u32> HwShift for Bitvector<L> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        if L == 0 {
            return self;
        }
        // shifting left logically
        let mut intervals = Vec::new();

        let large_amount = UnsignedBitvector::new(L as u64);

        for amount_interval in amount.unsigned_intervals() {
            // adjust too large amounts first
            let amount_interval = if amount_interval.max >= large_amount {
                // amount too large, add zero interval
                intervals.push(Interval::new(
                    UnsignedBitvector::zero(),
                    UnsignedBitvector::zero(),
                ));
                if amount_interval.min >= large_amount {
                    // any value from the amount interval will result in zero
                    continue;
                }
                Interval::new(amount_interval.min, UnsignedBitvector::new((L - 1) as u64))
            } else {
                amount_interval
            };

            for amount in amount_interval.iter() {
                // we will multiply bound diff by 2^amount and see if this results in interval overflow
                let multiplier = UnsignedBitvector::one() << amount;
                if self
                    .bound_diff()
                    .checked_mul(multiplier.as_bitvector())
                    .is_some()
                {
                    // not full
                    let start = UnsignedBitvector::from_bitvector(self.start) << amount;
                    let end = UnsignedBitvector::from_bitvector(self.end) << amount;
                    let result =
                        Bitvector::from_wrap_interval(start.as_bitvector(), end.as_bitvector());
                    intervals.extend(result.unsigned_intervals());
                } else {
                    // can be full
                    return Self::full();
                }
            }
        }

        Self::from_unsigned_intervals(intervals)
    }

    fn logic_shr(self, amount: Self) -> Self {
        if L == 0 {
            return self;
        }
        // shifting right logically
        let mut intervals = Vec::new();

        let large_amount = UnsignedBitvector::new(L as u64);

        for amount_interval in amount.unsigned_intervals() {
            // adjust too large amounts first
            let amount_interval = if amount_interval.max >= large_amount {
                // amount too large, add zero interval
                intervals.push(Interval::new(
                    UnsignedBitvector::zero(),
                    UnsignedBitvector::zero(),
                ));
                if amount_interval.min >= large_amount {
                    // any value from the amount interval will result in zero
                    continue;
                }
                Interval::new(amount_interval.min, UnsignedBitvector::new((L - 1) as u64))
            } else {
                amount_interval
            };

            for amount in amount_interval.iter() {
                // look at all unsigned intervals
                for lhs_interval in self.unsigned_intervals() {
                    // no interval overflow possible, just shift right
                    let min = lhs_interval.min >> amount;
                    let max = lhs_interval.max >> amount;
                    intervals.push(Interval::new(min, max));
                }
            }
        }

        Self::from_unsigned_intervals(intervals)
    }

    fn arith_shr(self, amount: Self) -> Self {
        if L == 0 {
            return self;
        }
        // shifting right arithmetically
        let mut intervals = Vec::new();

        let large_amount = UnsignedBitvector::new(L as u64);

        for amount_interval in amount.unsigned_intervals() {
            // adjust too large amounts first
            let amount_interval = if amount_interval.max >= large_amount {
                // amount too large, add intervals that copy possible result signs
                if self.intersects(&Bitvector::from_wrap_interval(
                    Self::representable_smin(),
                    Self::representable_umax(),
                )) {
                    let all_ones = UnsignedBitvector::from_bitvector(
                        ConcreteBitvector::<L>::new(1).arith_neg(),
                    );
                    intervals.push(Interval::new(all_ones, all_ones));
                }
                if self.intersects(&Bitvector::from_wrap_interval(
                    Self::representable_umin(),
                    Self::representable_smax(),
                )) {
                    intervals.push(Interval::new(
                        UnsignedBitvector::zero(),
                        UnsignedBitvector::zero(),
                    ));
                }
                if amount_interval.min >= large_amount {
                    // any value from the amount interval is too large
                    continue;
                }
                Interval::new(amount_interval.min, UnsignedBitvector::new((L - 1) as u64))
            } else {
                amount_interval
            };

            for amount in amount_interval.iter() {
                // look at negative intervals first
                for lhs_interval in self.negative_intervals() {
                    // no interval overflow possible, just shift right
                    let min = lhs_interval
                        .min
                        .as_bitvector()
                        .arith_shr(amount.as_bitvector());
                    let max = lhs_interval
                        .max
                        .as_bitvector()
                        .arith_shr(amount.as_bitvector());
                    // it is possible that there will be wrap to zero
                    intervals.extend(Bitvector::from_wrap_interval(min, max).unsigned_intervals());
                }
                // look at nonnegative intervals next
                for lhs_interval in self.nonnegative_intervals() {
                    // no interval overflow possible, just shift right
                    let min = lhs_interval
                        .min
                        .as_bitvector()
                        .arith_shr(amount.as_bitvector());
                    let max = lhs_interval
                        .max
                        .as_bitvector()
                        .arith_shr(amount.as_bitvector());
                    intervals.push(Interval::new(
                        UnsignedBitvector::from_bitvector(min),
                        UnsignedBitvector::from_bitvector(max),
                    ));
                }
            }
        }

        Self::from_unsigned_intervals(intervals)
    }
}
