use crate::{
    abstr::Phi,
    bitvector::interval::{SignedInterval, SignlessInterval, UnsignedInterval, WrappingInterval},
    concr::ConcreteBitvector,
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
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct DualInterval<const W: u32> {
    // The interval usually located between (including) 0 and (2^N)/2-1.
    //
    // If it is not, it must be equal to the far half.
    near_half: SignlessInterval<W>,
    // The interval usually located between (including) (2^N)/2 and (2^N)-1.
    //
    // If it is not, it must be equal to the near half.
    far_half: SignlessInterval<W>,
}

pub use support::DualIntervalFieldValue;

impl<const W: u32> DualInterval<W> {
    pub const FULL: Self = Self {
        near_half: SignlessInterval::FULL_NEAR_HALFPLANE,
        far_half: SignlessInterval::FULL_FAR_HALFPLANE,
    };

    pub(crate) fn from_wrapping_intervals(intervals: &[WrappingInterval<W>]) -> Self {
        let mut near_half = None;
        let mut far_half = None;

        for interval in intervals {
            let (interval_near_half, interval_far_half) = halves::wrapping_halves(*interval);
            near_half = SignlessInterval::union_opt(near_half, interval_near_half);
            far_half = SignlessInterval::union_opt(far_half, interval_far_half);
        }

        Self::from_opt_halves(near_half, far_half)
    }

    fn from_unsigned_intervals(intervals: impl IntoIterator<Item = UnsignedInterval<W>>) -> Self {
        let mut near_half = None;
        let mut far_half = None;

        for interval in intervals {
            let (interval_near_half, interval_far_half) = halves::unsigned_halves(interval);
            near_half = SignlessInterval::union_opt(near_half, interval_near_half);
            far_half = SignlessInterval::union_opt(far_half, interval_far_half);
        }

        Self::from_opt_halves(near_half, far_half)
    }

    fn from_signed_intervals(intervals: &[SignedInterval<W>]) -> Self {
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

impl<const W: u32> Default for DualInterval<W> {
    fn default() -> Self {
        Self {
            near_half: SignlessInterval::from_value(ConcreteBitvector::zero()),
            far_half: SignlessInterval::from_value(ConcreteBitvector::zero()),
        }
    }
}

impl<const W: u32> Phi for DualInterval<W> {
    fn phi(self, other: Self) -> Self {
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

    fn uninit() -> Self {
        Self::default()
    }
}
