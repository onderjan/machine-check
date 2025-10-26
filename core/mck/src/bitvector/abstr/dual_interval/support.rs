use super::DualInterval;
use crate::{
    abstr::BitvectorDomain,
    bitvector::interval::{SignedInterval, SignlessInterval, UnsignedInterval, WrappingInterval},
    concr::ConcreteBitvector,
    misc::{BitvectorBound, MetaEq},
};

use std::fmt::{Debug, Display};

impl<B: BitvectorBound> MetaEq for DualInterval<B> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.near_half == other.near_half && self.far_half == other.far_half
    }
}

impl<B: BitvectorBound> DualInterval<B> {
    pub fn bound(&self) -> B {
        // bound must be the same for near half and far half
        self.near_half.bound()
    }

    pub fn from_value(value: ConcreteBitvector<B>) -> Self {
        Self {
            near_half: SignlessInterval::from_value(value),
            far_half: SignlessInterval::from_value(value),
        }
    }

    pub fn contains_value(&self, value: &ConcreteBitvector<B>) -> bool {
        self.near_half.contains_value(value) || self.far_half.contains_value(value)
    }

    pub fn to_unsigned_interval(self) -> UnsignedInterval<B> {
        UnsignedInterval::new(self.umin(), self.umax())
    }

    pub fn to_signed_interval(self) -> SignedInterval<B> {
        SignedInterval::new(self.smin(), self.smax())
    }

    pub(super) fn resolve_by_wrapping(
        a: DualInterval<B>,
        b: DualInterval<B>,
        op_fn: fn(WrappingInterval<B>, WrappingInterval<B>) -> WrappingInterval<B>,
    ) -> DualInterval<B> {
        assert_eq!(a.bound(), b.bound());
        // TODO: optimise cases where the a, b, or both can be represented by one wrapping interval

        // resolve all combinations of halves separately
        let nn_result = op_fn(a.near_half.into_wrapping(), b.near_half.into_wrapping());
        let nf_result = op_fn(a.near_half.into_wrapping(), b.far_half.into_wrapping());
        let fn_result = op_fn(a.far_half.into_wrapping(), b.near_half.into_wrapping());
        let ff_result = op_fn(a.far_half.into_wrapping(), b.far_half.into_wrapping());

        DualInterval::from_wrapping_intervals(&[nn_result, nf_result, fn_result, ff_result])
    }

    pub(super) fn resolve_by_unsigned(
        a: DualInterval<B>,
        b: DualInterval<B>,
        op_fn: fn(UnsignedInterval<B>, UnsignedInterval<B>) -> UnsignedInterval<B>,
    ) -> DualInterval<B> {
        assert_eq!(a.bound(), b.bound());
        // TODO: optimise cases where the a, b, or both can be represented by one wrapping interval

        // resolve all combinations of halves separately
        let nn_result = op_fn(a.near_half.into_unsigned(), b.near_half.into_unsigned());
        let nf_result = op_fn(a.near_half.into_unsigned(), b.far_half.into_unsigned());
        let fn_result = op_fn(a.far_half.into_unsigned(), b.near_half.into_unsigned());
        let ff_result = op_fn(a.far_half.into_unsigned(), b.far_half.into_unsigned());

        DualInterval::from_unsigned_intervals([nn_result, nf_result, fn_result, ff_result])
    }
}

impl<B: BitvectorBound> Debug for DualInterval<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_interval<B: BitvectorBound>(
            f: &mut std::fmt::Formatter<'_>,
            min: ConcreteBitvector<B>,
            max: ConcreteBitvector<B>,
        ) -> std::fmt::Result {
            if min == max {
                write!(f, "{}", min)
            } else {
                write!(f, "[{}, {}]", min, max)
            }
        }

        let near_min = self.near_half.min();
        let near_max = self.near_half.max();
        let far_min = self.far_half.min();
        let far_max = self.far_half.max();

        if near_min == far_min && near_max == far_max {
            // write just one interval
            write_interval(f, near_min, near_max)?;
        } else if near_max
            .checked_add(ConcreteBitvector::new(1, self.bound()))
            .map(|above_near_max| above_near_max == far_min)
            .unwrap_or(false)
        {
            // the near half directly precedes and touches the far half
            // write them together
            write_interval(f, near_min, far_max)?;
        } else {
            // write the union of two intervals
            write!(f, "(")?;
            write_interval(f, near_min, near_max)?;
            write!(f, " ∪ ")?;
            write_interval(f, far_min, far_max)?;
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl<B: BitvectorBound> Display for DualInterval<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
