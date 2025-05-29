use machine_check_common::{PANIC_NUM_DIV_BY_ZERO, PANIC_NUM_NO_PANIC, PANIC_NUM_REM_BY_ZERO};

use crate::{
    abstr::{self, PanicResult, Phi},
    bitvector::concrete::{ConcreteBitvector, UnsignedInterval},
    forward::HwArith,
};

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
        let result = resolve_by_unsigned(self, rhs, |a, b| a.hw_udiv(b));
        let zero = ConcreteBitvector::zero();
        let may_panic = rhs.contains_value(&zero);
        let must_panic = rhs.concrete_value() == Some(zero);
        construct_panic_result(result, may_panic, must_panic, PANIC_NUM_DIV_BY_ZERO)
    }

    fn sdiv(self, rhs: Self) -> PanicResult<Self> {
        // TODO: compute using signed intervals
        todo!()
    }

    fn urem(self, rhs: Self) -> PanicResult<Self> {
        let result = resolve_by_unsigned(self, rhs, |a, b| a.hw_urem(b));
        let zero = ConcreteBitvector::zero();
        let may_panic = rhs.contains_value(&zero);
        let must_panic = rhs.concrete_value() == Some(zero);
        construct_panic_result(result, may_panic, must_panic, PANIC_NUM_REM_BY_ZERO)
    }

    fn srem(self, rhs: Self) -> PanicResult<Self> {
        // TODO: compute using signed intervals
        todo!()
    }
}

fn construct_panic_result<T>(
    result: T,
    may_panic: bool,
    must_panic: bool,
    panic_msg_num: u64,
) -> PanicResult<T> {
    let panic = if must_panic {
        abstr::Bitvector::new(panic_msg_num)
    } else if may_panic {
        abstr::Bitvector::new(PANIC_NUM_NO_PANIC).phi(abstr::Bitvector::new(panic_msg_num))
    } else {
        abstr::Bitvector::new(PANIC_NUM_NO_PANIC)
    };
    PanicResult { panic, result }
}

fn resolve_by_wrapping<const W: u32>(
    a: DualInterval<W>,
    b: DualInterval<W>,
    op_fn: fn(WrappingInterval<W>, WrappingInterval<W>) -> WrappingInterval<W>,
) -> DualInterval<W> {
    println!("Resolving by wrapping for a: {}, b: {}", a, b);

    // TODO: optimise cases where the a, b, or both can be represented by one wrapping interval

    // resolve all combinations of halves separately
    let nn_result = op_fn(a.near_half.into_wrapping(), b.near_half.into_wrapping());
    let nf_result = op_fn(a.near_half.into_wrapping(), b.far_half.into_wrapping());
    let fn_result = op_fn(a.far_half.into_wrapping(), b.near_half.into_wrapping());
    let ff_result = op_fn(a.far_half.into_wrapping(), b.far_half.into_wrapping());

    DualInterval::from_wrapping_intervals(&[nn_result, nf_result, fn_result, ff_result])
}

fn resolve_by_unsigned<const W: u32>(
    a: DualInterval<W>,
    b: DualInterval<W>,
    op_fn: fn(UnsignedInterval<W>, UnsignedInterval<W>) -> UnsignedInterval<W>,
) -> DualInterval<W> {
    println!("Resolving by unsigned for a: {}, b: {}", a, b);

    // TODO: optimise cases where the a, b, or both can be represented by one wrapping interval

    // resolve all combinations of halves separately
    let nn_result = op_fn(a.near_half.into_unsigned(), b.near_half.into_unsigned());
    let nf_result = op_fn(a.near_half.into_unsigned(), b.far_half.into_unsigned());
    let fn_result = op_fn(a.far_half.into_unsigned(), b.near_half.into_unsigned());
    let ff_result = op_fn(a.far_half.into_unsigned(), b.far_half.into_unsigned());

    DualInterval::from_unsigned_intervals(&[nn_result, nf_result, fn_result, ff_result])
}
