use super::DualInterval;
use crate::{
    concr::{
        ConcreteBitvector, SignedBitvector, SignedInterval, SignlessInterval, UnsignedBitvector,
        UnsignedInterval, WrappingInterval,
    },
    misc::MetaEq,
};

use std::fmt::{Debug, Display};

impl<const W: u32> MetaEq for DualInterval<W> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.near_half == other.near_half && self.far_half == other.far_half
    }
}

impl<const W: u32> DualInterval<W> {
    pub fn from_value(value: ConcreteBitvector<W>) -> Self {
        Self {
            near_half: SignlessInterval::from_value(value),
            far_half: SignlessInterval::from_value(value),
        }
    }

    pub fn contains_value(&self, value: &ConcreteBitvector<W>) -> bool {
        self.near_half.contains_value(value) || self.far_half.contains_value(value)
    }

    pub fn concrete_value(&self) -> Option<ConcreteBitvector<W>> {
        if self.near_half == self.far_half {
            return self.near_half.concrete_value();
        }
        None
    }

    pub(super) fn opt_halves(self) -> (Option<SignlessInterval<W>>, Option<SignlessInterval<W>>) {
        if self.near_half == self.far_half {
            if self.near_half.is_sign_bit_set() {
                (None, Some(self.far_half))
            } else {
                (Some(self.near_half), None)
            }
        } else {
            (Some(self.near_half), Some(self.far_half))
        }
    }

    pub fn intersection(&self, rhs: &Self) -> Option<Self> {
        let (our_near_half, our_far_half) = self.opt_halves();
        let (other_near_half, other_far_half) = rhs.opt_halves();

        let mut result_near_half = None;
        let mut result_far_half = None;

        if let (Some(our_near_half), Some(other_near_half)) = (our_near_half, other_near_half) {
            result_near_half = our_near_half.intersection(other_near_half);
        }

        if let (Some(our_far_half), Some(other_far_half)) = (our_far_half, other_far_half) {
            result_far_half = our_far_half.intersection(other_far_half);
        }

        Self::try_from_opt_halves(result_near_half, result_far_half)
    }

    pub fn unsigned_min(&self) -> UnsignedBitvector<W> {
        self.near_half.min().cast_unsigned()
    }

    pub fn unsigned_max(&self) -> UnsignedBitvector<W> {
        self.far_half.max().cast_unsigned()
    }

    pub fn to_unsigned_interval(self) -> UnsignedInterval<W> {
        UnsignedInterval::new(self.unsigned_min(), self.unsigned_max())
    }

    pub fn signed_min(&self) -> SignedBitvector<W> {
        self.far_half.min().cast_signed()
    }

    pub fn signed_max(&self) -> SignedBitvector<W> {
        self.near_half.max().cast_signed()
    }

    pub fn to_signed_interval(self) -> SignedInterval<W> {
        SignedInterval::new(self.signed_min(), self.signed_max())
    }

    pub(super) fn resolve_by_wrapping(
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

    pub(super) fn resolve_by_unsigned(
        a: DualInterval<W>,
        b: DualInterval<W>,
        op_fn: fn(UnsignedInterval<W>, UnsignedInterval<W>) -> UnsignedInterval<W>,
    ) -> DualInterval<W> {
        // TODO: optimise cases where the a, b, or both can be represented by one wrapping interval

        // resolve all combinations of halves separately
        let nn_result = op_fn(a.near_half.into_unsigned(), b.near_half.into_unsigned());
        let nf_result = op_fn(a.near_half.into_unsigned(), b.far_half.into_unsigned());
        let fn_result = op_fn(a.far_half.into_unsigned(), b.near_half.into_unsigned());
        let ff_result = op_fn(a.far_half.into_unsigned(), b.far_half.into_unsigned());

        DualInterval::from_unsigned_intervals([nn_result, nf_result, fn_result, ff_result])
    }
}

impl<const W: u32> Debug for DualInterval<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.near_half == self.far_half {
            write!(f, "{}", self.near_half)
        } else {
            write!(f, "{} ⊔ {}", self.near_half, self.far_half)
        }
    }
}

impl<const W: u32> Display for DualInterval<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
