use num::{
    traits::{WrappingAdd, WrappingMul, WrappingNeg, WrappingSub},
    PrimInt,
};

use super::concrete::{
    ConcreteBitvector, SignedInterval, SignlessInterval, UnsignedInterval, WrappingInterpretation,
    WrappingInterval,
};

mod arith;
mod cmp;
mod eq;
mod support;

#[cfg(test)]
mod tests;

trait UnsignedPrimitive: PrimInt + WrappingAdd + WrappingSub + WrappingMul + WrappingNeg {
    type Signed: SignedPrimitive;

    const ZERO: Self;
    const UNDERHALF: Self;
    const OVERHALF: Self;
    const MAX: Self;

    fn cast_signed(self) -> Self::Signed;
}
trait SignedPrimitive: PrimInt + WrappingAdd + WrappingSub + WrappingMul + WrappingNeg {
    type Unsigned: UnsignedPrimitive;
    fn cast_unsigned(self) -> Self::Unsigned;
}

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
pub(crate) struct DualInterval<const W: u32> {
    // The interval usually located between (including) 0 and (2^N)/2-1.
    //
    // If it is not, it must be equal to the far half.
    near_half: SignlessInterval<W>,
    // The interval usually located between (including) (2^N)/2 and (2^N)-1.
    //
    // If it is not, it must be equal to the near half.
    far_half: SignlessInterval<W>,
}

impl<const W: u32> DualInterval<W> {
    pub const FULL: Self = Self {
        near_half: SignlessInterval::FULL_NEAR_HALFPLANE,
        far_half: SignlessInterval::FULL_FAR_HALFPLANE,
    };

    fn from_wrapping_interval(a: WrappingInterval<W>) -> Self {
        let (near_half, far_half) = wrapping_halves(a);
        Self::from_opt_halves(near_half, far_half)
    }

    fn from_wrapping_intervals(intervals: &[WrappingInterval<W>]) -> Self {
        let mut near_half = None;
        let mut far_half = None;

        for interval in intervals {
            let (interval_near_half, interval_far_half) = wrapping_halves(*interval);
            near_half = SignlessInterval::union_opt(near_half, interval_near_half);
            far_half = SignlessInterval::union_opt(far_half, interval_far_half);
        }

        Self::from_opt_halves(near_half, far_half)
    }

    fn from_unsigned_intervals(intervals: &[UnsignedInterval<W>]) -> Self {
        let mut near_half = None;
        let mut far_half = None;

        for interval in intervals {
            let (interval_near_half, interval_far_half) = unsigned_halves(*interval);
            near_half = SignlessInterval::union_opt(near_half, interval_near_half);
            far_half = SignlessInterval::union_opt(far_half, interval_far_half);
        }

        Self::from_opt_halves(near_half, far_half)
    }

    fn from_signed_intervals(intervals: &[SignedInterval<W>]) -> Self {
        let mut near_half = None;
        let mut far_half = None;

        for interval in intervals {
            let (interval_near_half, interval_far_half) = signed_halves(*interval);
            near_half = SignlessInterval::union_opt(near_half, interval_near_half);
            far_half = SignlessInterval::union_opt(far_half, interval_far_half);
        }

        Self::from_opt_halves(near_half, far_half)
    }

    fn try_from_opt_halves(
        near_half: Option<SignlessInterval<W>>,
        far_half: Option<SignlessInterval<W>>,
    ) -> Option<Self> {
        // if one is not present, take the other
        let (near_half, far_half) = match (near_half, far_half) {
            (None, None) => return None,
            (None, Some(far_half)) => (far_half, far_half),
            (Some(near_half), None) => (near_half, near_half),
            (Some(near_half), Some(far_half)) => (near_half, far_half),
        };

        Some(Self {
            near_half,
            far_half,
        })
    }

    fn from_opt_halves(
        near_half: Option<SignlessInterval<W>>,
        far_half: Option<SignlessInterval<W>>,
    ) -> Self {
        Self::try_from_opt_halves(near_half, far_half)
            .expect("At least one half should be supplied")
    }
}

fn wrapping_halves<const W: u32>(
    interval: WrappingInterval<W>,
) -> (Option<SignlessInterval<W>>, Option<SignlessInterval<W>>) {
    let interpreted = interval.interpret();

    match interpreted {
        WrappingInterpretation::Signless(interval) => {
            let far_half = interval.min().is_sign_bit_set();
            if far_half {
                (None, Some(interval))
            } else {
                (Some(interval), None)
            }
        }
        WrappingInterpretation::Unsigned(interval) => (
            Some(SignlessInterval::new(
                interval.min().as_bitvector(),
                ConcreteBitvector::<W>::const_underhalf(),
            )),
            Some(SignlessInterval::new(
                ConcreteBitvector::<W>::const_overhalf(),
                interval.max().as_bitvector(),
            )),
        ),
        WrappingInterpretation::Signed(interval) => (
            Some(SignlessInterval::new(
                ConcreteBitvector::<W>::zero(),
                interval.max().as_bitvector(),
            )),
            Some(SignlessInterval::new(
                interval.min().as_bitvector(),
                ConcreteBitvector::<W>::const_umax(),
            )),
        ),
    }
}

fn unsigned_halves<const W: u32>(
    interval: UnsignedInterval<W>,
) -> (Option<SignlessInterval<W>>, Option<SignlessInterval<W>>) {
    match interval.try_into_signless() {
        Some(interval) => {
            let far_half = interval.is_sign_bit_set();
            if far_half {
                (None, Some(interval))
            } else {
                (Some(interval), None)
            }
        }
        None => (
            Some(SignlessInterval::new(
                interval.min().as_bitvector(),
                ConcreteBitvector::<W>::const_underhalf(),
            )),
            Some(SignlessInterval::new(
                ConcreteBitvector::<W>::const_overhalf(),
                interval.max().as_bitvector(),
            )),
        ),
    }
}

fn signed_halves<const W: u32>(
    interval: SignedInterval<W>,
) -> (Option<SignlessInterval<W>>, Option<SignlessInterval<W>>) {
    match interval.try_into_signless() {
        Some(interval) => {
            let far_half = interval.is_sign_bit_set();
            if far_half {
                (None, Some(interval))
            } else {
                (Some(interval), None)
            }
        }
        None => (
            Some(SignlessInterval::new(
                ConcreteBitvector::<W>::zero(),
                interval.max().as_bitvector(),
            )),
            Some(SignlessInterval::new(
                interval.min().as_bitvector(),
                ConcreteBitvector::<W>::const_umax(),
            )),
        ),
    }
}
