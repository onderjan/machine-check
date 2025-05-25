use crate::{abstr::PanicResult, forward::HwArith};

use super::{DualInterval, UnsignedPrimitive, WrappingInterval};

impl<U: UnsignedPrimitive> WrappingInterval<U> {
    fn hw_add(self, rhs: Self) -> Self {
        // ensure the produced bounds are less than 2^L apart, produce a full interval otherwise
        if self.is_addsub_full(rhs) {
            Self::full()
        } else {
            // wrapping and fully monotonic: add bounds
            let start = self.start.add(rhs.start);
            let end = self.end.add(rhs.end);

            Self { start, end }
        }
    }

    fn hw_sub(self, rhs: Self) -> Self {
        // ensure the produced bounds are less than 2^L apart, produce a full interval otherwise
        if self.is_addsub_full(rhs) {
            Self::full()
        } else {
            // wrapping, monotonic on lhs, anti-monotonic on rhs: subtract bounds, remember to flip rhs bounds
            let start = self.start.sub(rhs.end);
            let end = self.end.sub(rhs.start);

            Self { start, end }
        }
    }

    fn hw_mul(self, rhs: Self) -> Self {
        let lhs_start = self.start;
        let rhs_start = rhs.start;
        let start = lhs_start.wrapping_mul(&rhs_start);
        let lhs_diff = self.bound_diff();
        let rhs_diff = rhs.bound_diff();

        let Some(diff_product) = lhs_diff.checked_mul(&rhs_diff) else {
            return Self::full();
        };
        let Some(diff_start_product) = lhs_diff.checked_mul(&rhs_start) else {
            return Self::full();
        };
        let Some(start_diff_product) = lhs_start.checked_mul(&rhs_diff) else {
            return Self::full();
        };
        let Some(result_len) = diff_product
            .checked_add(&diff_start_product)
            .and_then(|v| v.checked_add(&start_diff_product))
        else {
            return Self::full();
        };

        let end = start.wrapping_add(&result_len);

        Self { start, end }
    }

    fn is_addsub_full(self, rhs: Self) -> bool {
        let lhs_diff = self.bound_diff();
        let rhs_diff = rhs.bound_diff();

        let wrapped_total_len = lhs_diff.wrapping_add(&rhs_diff);
        wrapped_total_len < lhs_diff || wrapped_total_len < rhs_diff
    }

    pub fn bound_diff(&self) -> U {
        self.end.wrapping_sub(&self.start)
    }
}

impl<U: UnsignedPrimitive> HwArith for DualInterval<U> {
    type DivRemResult = PanicResult<Self>;

    fn arith_neg(self) -> Self {
        // arithmetic negation
        // for wrapping arithmetic, arithmetic negation is same as subtracting the value from 0
        // subtract from interval constructed from 0
        Self::from_value(U::zero()).sub(self)
    }

    fn add(self, rhs: Self) -> Self {
        resolve_by_wrapping(self, rhs, |a, b| a.hw_add(b))
    }

    fn sub(self, rhs: Self) -> Self {
        resolve_by_wrapping(self, rhs, |a, b| a.hw_sub(b))
    }

    fn mul(self, rhs: Self) -> Self {
        resolve_by_wrapping(self, rhs, |a, b| a.hw_mul(b))
    }

    fn udiv(self, rhs: Self) -> PanicResult<Self> {
        // TODO: compute using unsigned intervals
        todo!()
    }

    fn sdiv(self, rhs: Self) -> PanicResult<Self> {
        // TODO: compute using signed intervals
        todo!()
    }

    fn urem(self, rhs: Self) -> PanicResult<Self> {
        // TODO: compute using unsigned intervals
        todo!()
    }

    fn srem(self, rhs: Self) -> PanicResult<Self> {
        // TODO: compute using signed intervals
        todo!()
    }
}

fn resolve_by_wrapping<U: UnsignedPrimitive>(
    a: DualInterval<U>,
    b: DualInterval<U>,
    op_fn: fn(WrappingInterval<U>, WrappingInterval<U>) -> WrappingInterval<U>,
) -> DualInterval<U> {
    // TODO: optimise cases where the a, b, or both can be represented by one wrapping interval

    // resolve all combinations of halves separately
    let nn_result = op_fn(a.near_half.into_wrapping(), b.near_half.into_wrapping());
    let nf_result = op_fn(a.near_half.into_wrapping(), b.far_half.into_wrapping());
    let fn_result = op_fn(a.far_half.into_wrapping(), b.near_half.into_wrapping());
    let ff_result = op_fn(a.far_half.into_wrapping(), b.far_half.into_wrapping());

    DualInterval::from_wrapping_intervals(&[nn_result, nf_result, fn_result, ff_result])
}
