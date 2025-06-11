use crate::{
    bitvector::abstr::dual_interval::DualInterval,
    concr::{
        ConcreteBitvector, SignedInterval, SignlessInterval, UnsignedInterval,
        WrappingInterpretation, WrappingInterval,
    },
};

impl<const W: u32> DualInterval<W> {
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

    pub(super) fn try_from_opt_halves(
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

    pub(super) fn from_opt_halves(
        near_half: Option<SignlessInterval<W>>,
        far_half: Option<SignlessInterval<W>>,
    ) -> Self {
        Self::try_from_opt_halves(near_half, far_half)
            .expect("At least one half should be supplied")
    }
}

pub fn wrapping_halves<const W: u32>(
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

pub fn unsigned_halves<const W: u32>(
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

pub fn signed_halves<const W: u32>(
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
