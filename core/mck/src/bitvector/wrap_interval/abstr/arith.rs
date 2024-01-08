use super::Bitvector;
use crate::{bitvector::concrete::ConcreteBitvector, forward::HwArith};

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
        let lhs_interval = self.unsigned_interval();
        let rhs_interval = rhs.unsigned_interval();

        let result_max =
            ConcreteBitvector::new(lhs_interval.max).udiv(ConcreteBitvector::new(rhs_interval.min));
        let result_min =
            ConcreteBitvector::new(lhs_interval.min).udiv(ConcreteBitvector::new(rhs_interval.max));

        let start = result_min;
        let end = result_max;

        Self { start, end }
    }

    fn sdiv(self, rhs: Self) -> Self {
        todo!()
    }

    fn urem(self, rhs: Self) -> Self {
        todo!()
    }

    fn srem(self, rhs: Self) -> Self {
        todo!()
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
}
