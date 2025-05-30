use crate::{
    abstr::Phi,
    concr::{
        ConcreteBitvector, SignedInterval, SignlessInterval, UnsignedInterval,
        WrappingInterpretation, WrappingInterval,
    },
};

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
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

    pub(crate) fn from_wrapping_intervals(intervals: &[WrappingInterval<W>]) -> Self {
        let mut near_half = None;
        let mut far_half = None;

        for interval in intervals {
            let (interval_near_half, interval_far_half) = wrapping_halves(*interval);
            near_half = SignlessInterval::union_opt(near_half, interval_near_half);
            far_half = SignlessInterval::union_opt(far_half, interval_far_half);
        }

        Self::from_opt_halves(near_half, far_half)
    }

    fn from_unsigned_intervals(intervals: impl IntoIterator<Item = UnsignedInterval<W>>) -> Self {
        let mut near_half = None;
        let mut far_half = None;

        for interval in intervals {
            let (interval_near_half, interval_far_half) = unsigned_halves(interval);
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

        let mut result_near_half = None;
        let mut result_far_half = None;

        if let (Some(our_near_half), Some(other_near_half)) = (our_near_half, other_near_half) {
            result_near_half = Some(our_near_half.union(other_near_half));
        }

        if let (Some(our_far_half), Some(other_far_half)) = (our_far_half, other_far_half) {
            result_far_half = Some(our_far_half.union(other_far_half));
        }

        Self::from_opt_halves(result_near_half, result_far_half)
    }

    fn uninit() -> Self {
        Self::default()
    }
}
