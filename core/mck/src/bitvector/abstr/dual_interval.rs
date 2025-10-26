use crate::{
    abstr::{BitvectorDomain, CBitvectorDomain},
    bitvector::interval::{SignedInterval, SignlessInterval, UnsignedInterval, WrappingInterval},
    concr::{CConcreteBitvector, ConcreteBitvector, SignedBitvector, UnsignedBitvector},
    misc::{BitvectorBound, CBound, Join, RBound},
};

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;

mod halves;
mod support;

#[cfg(test)]
mod tests;

/// Dual-interval domain.
///
/// The idea is that the signedness of operations on the variable only really
/// impacts the continuity of the highest bit: the wrapping point is located
/// between -1 and 0 for unsigned, and between (2^N)/2-1 and (2^N)/2 for signed.
/// As such, we will consider the halves completely separately with distinct
/// intervals for each half.
///
/// The near half is located between (including) 0 and (2^N)/2-1 when interpreted
/// both as unsigned and signed. The far half is located between (including)
/// (2^N)/2 and (2^N)-1 when interpreted as unsigned and betweeen (including)
/// -(2^N)/2 and -1 when considered as signed in two's complement.
///
/// The only exception is when one of the halves does not have any value present,
/// in which case both intervals will be set equal to each other.
///
/// Unlike wrapping intervals in general, this domain forms a lattice,
/// since each half can be thought about as selecting elements from its own half
/// but admitting every element from the other half, and this domain is
/// their product (in the meaning of abstract interpretation). We pay for the
/// increased precision compared to a standard interval domain (which forms a lattice)
/// or a wrapping-interval domain (which does not, but can be more precise than
/// a non-wrapping interval) by an increase in time and memory, which should not
/// be problematic for our use.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualInterval<B: BitvectorBound> {
    // The interval usually located between (including) 0 and (2^N)/2-1.
    //
    // If it is not, it must be equal to the far half.
    near_half: SignlessInterval<B>,
    // The interval usually located between (including) (2^N)/2 and (2^N)-1.
    //
    // If it is not, it must be equal to the near half.
    far_half: SignlessInterval<B>,
}

pub type CDualInterval<const W: u32> = DualInterval<CBound<W>>;
pub type RDualInterval = DualInterval<RBound>;

use serde::{Deserialize, Serialize};

impl<B: BitvectorBound> DualInterval<B> {
    pub fn new(value: u64, bound: B) -> Self {
        Self::from_value(ConcreteBitvector::new(value, bound))
    }

    pub(crate) fn from_wrapping_intervals(intervals: &[WrappingInterval<B>]) -> Self {
        let mut near_half = None;
        let mut far_half = None;

        for interval in intervals {
            let (interval_near_half, interval_far_half) = halves::wrapping_halves(*interval);
            near_half = SignlessInterval::union_opt(near_half, interval_near_half);
            far_half = SignlessInterval::union_opt(far_half, interval_far_half);
        }

        Self::from_opt_halves(near_half, far_half)
    }

    fn from_unsigned_intervals(intervals: impl IntoIterator<Item = UnsignedInterval<B>>) -> Self {
        let mut near_half = None;
        let mut far_half = None;

        for interval in intervals {
            let (interval_near_half, interval_far_half) = halves::unsigned_halves(interval);
            near_half = SignlessInterval::union_opt(near_half, interval_near_half);
            far_half = SignlessInterval::union_opt(far_half, interval_far_half);
        }

        Self::from_opt_halves(near_half, far_half)
    }

    fn from_signed_intervals(intervals: &[SignedInterval<B>]) -> Self {
        let mut near_half = None;
        let mut far_half = None;

        for interval in intervals {
            let (interval_near_half, interval_far_half) = halves::signed_halves(*interval);
            near_half = SignlessInterval::union_opt(near_half, interval_near_half);
            far_half = SignlessInterval::union_opt(far_half, interval_far_half);
        }

        Self::from_opt_halves(near_half, far_half)
    }
}

impl<B: BitvectorBound> Join for DualInterval<B> {
    fn join(self, other: &Self) -> Self {
        let (our_near_half, our_far_half) = self.opt_halves();
        let (other_near_half, other_far_half) = other.opt_halves();

        let result_near_half = match (our_near_half, other_near_half) {
            (None, None) => None,
            (None, Some(half)) => Some(half),
            (Some(half), None) => Some(half),
            (Some(our_half), Some(other_half)) => Some(our_half.union(other_half)),
        };

        let result_far_half = match (our_far_half, other_far_half) {
            (None, None) => None,
            (None, Some(half)) => Some(half),
            (Some(half), None) => Some(half),
            (Some(our_half), Some(other_half)) => Some(our_half.union(other_half)),
        };

        Self::from_opt_halves(result_near_half, result_far_half)
    }
}

impl<B: BitvectorBound> BitvectorDomain for DualInterval<B> {
    type Bound = B;
    type General<X: BitvectorBound> = DualInterval<X>;

    fn single_value(value: ConcreteBitvector<B>) -> Self {
        Self::from_value(value)
    }

    fn top(bound: Self::Bound) -> Self {
        Self {
            near_half: SignlessInterval::new_full_near_halfplane(bound),
            far_half: SignlessInterval::new_full_far_halfplane(bound),
        }
    }

    fn bound(&self) -> Self::Bound {
        // both bounds must be the same
        self.near_half.bound()
    }

    fn meet(self, rhs: &Self) -> Option<Self> {
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

    fn umin(&self) -> UnsignedBitvector<B> {
        self.near_half.min().as_unsigned()
    }

    fn umax(&self) -> UnsignedBitvector<B> {
        self.far_half.max().as_unsigned()
    }

    fn smin(&self) -> SignedBitvector<B> {
        self.far_half.min().as_signed()
    }

    fn smax(&self) -> SignedBitvector<B> {
        self.near_half.max().as_signed()
    }

    fn concrete_value(&self) -> Option<ConcreteBitvector<Self::Bound>> {
        if self.near_half == self.far_half {
            return self.near_half.concrete_value();
        }
        None
    }
}

impl<const W: u32> CBitvectorDomain for CDualInterval<W> {
    type Concrete = CConcreteBitvector<W>;

    fn from_concrete_bitvector(value: Self::Concrete) -> Self {
        // just convert it to a signless interval and put to both halves
        let interval = SignlessInterval::from_value(value);

        Self {
            near_half: interval,
            far_half: interval,
        }
    }

    fn from_runtime_bitvector(value: Self::General<RBound>) -> Self {
        Self {
            near_half: SignlessInterval::from_runtime(value.near_half),
            far_half: SignlessInterval::from_runtime(value.far_half),
        }
    }

    fn as_runtime_bitvector(&self) -> Self::General<RBound> {
        Self::General {
            near_half: self.near_half.to_runtime(),
            far_half: self.far_half.to_runtime(),
        }
    }
}
