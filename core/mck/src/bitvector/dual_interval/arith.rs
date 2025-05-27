use crate::{abstr::PanicResult, bitvector::concrete::ConcreteBitvector, forward::HwArith};

use super::{DualInterval, WrappingInterval};

impl<const W: u32> HwArith for DualInterval<W> {
    type DivRemResult = PanicResult<Self>;

    fn arith_neg(self) -> Self {
        // arithmetic negation
        // for wrapping arithmetic, arithmetic negation is same as subtracting the value from 0
        // subtract from interval constructed from 0
        Self::from_value(ConcreteBitvector::zero()).sub(self)
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

fn resolve_by_wrapping<const W: u32>(
    a: DualInterval<W>,
    b: DualInterval<W>,
    op_fn: fn(WrappingInterval<W>, WrappingInterval<W>) -> WrappingInterval<W>,
) -> DualInterval<W> {
    // TODO: optimise cases where the a, b, or both can be represented by one wrapping interval

    // resolve all combinations of halves separately
    let nn_result = op_fn(a.near_half.into_wrapping(), b.near_half.into_wrapping());
    let nf_result = op_fn(a.near_half.into_wrapping(), b.far_half.into_wrapping());
    let fn_result = op_fn(a.far_half.into_wrapping(), b.near_half.into_wrapping());
    let ff_result = op_fn(a.far_half.into_wrapping(), b.far_half.into_wrapping());

    DualInterval::from_wrapping_intervals(&[nn_result, nf_result, fn_result, ff_result])
}
